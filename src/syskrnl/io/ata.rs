use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

use crate::{debugln, syskrnl};

/// ATA设备的块大小
pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_BIN_SZ: usize = 9;
pub const BLOCK_MASK: usize = 0x1FF;

/// ATA 设备支持的命令类型
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
}

enum IdentifyResponse {
    Ata([u16; 256]),
    Atapi,
    Sata,
    None,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum Status {
    // 错误
    ERR = 0,
    // （已过时）
    IDX = 1,
    // （已过时）
    CORR = 2,
    // 数据请求
    DRQ = 3,
    // （与命令相关）
    DSC = 4,
    // （与命令相关）
    DF = 5,
    // 设备就绪
    DRDY = 6,
    // 忙碌
    BSY = 7,
}

pub struct Bus {
    id: u8,
    irq: u8,

    data_register: Port<u16>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,

    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockess_register: PortReadOnly<u8>,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        // 参见ATA规范
        Self {
            id,
            irq,

            data_register: Port::new(io_base + 0),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),

            alternate_status_register: PortReadOnly::new(ctrl_base + 0),
            control_register: PortWriteOnly::new(ctrl_base + 0),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    /// 读取总线状态寄存器
    fn status(&mut self) -> u8 {
        unsafe { self.alternate_status_register.read() }
    }

    /// 检查总线是否处于浮空状态
    fn check_floating_bus(&mut self) -> Result<(), ()> {
        match self.status() {
            0xFF | 0x7F => Err(()),
            _ => Ok(()),
        }
    }

    /// 稍等
    fn wait(&mut self, ns: u64) {
        syskrnl::time::nanowait(ns);
    }

    /// 清除中断标志
    fn clear_interrupt(&mut self) -> u8 {
        unsafe { self.status_register.read() }
    }

    /// 读取LBA0寄存器（指示要读取或写入的扇区号的中 8 位）
    fn lba0(&mut self) -> u8 {
        unsafe { self.lba0_register.read() }
    }

    /// 读取LBA1寄存器（指示要读取或写入的扇区号的中 8 位）
    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    /// 读取LBA1寄存器（指示要读取或写入的扇区号的高 8 位）
    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    /// 从 ATA 总线上读取数据寄存器的值
    fn read_data(&mut self) -> u16 {
        unsafe { self.data_register.read() }
    }

    /// 向 ATA 总线上的数据寄存器写入数据
    fn write_data(&mut self, data: u16) {
        unsafe { self.data_register.write(data) }
    }

    /// 检查 ATA 总线的状态寄存器中是否有错误标志位被设置
    #[allow(clippy::wrong_self_convention)]
    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    /// 轮询 ATA 总线的状态寄存器，直到指定的标志位被设置为指定的值
    fn poll(&mut self, bit: Status, val: bool) -> Result<(), ()> {
        let start = syskrnl::time::uptime();
        while self.status().get_bit(bit as usize) != val {
            if syskrnl::time::uptime() - start > 1.0 {
                debugln!("ATA hanged while polling {:?} bit in status register", bit);
                self.debug();
                return Err(());
            }
            core::hint::spin_loop();
        }
        Ok(())
    }

    /// 选择ATA总线上的指定驱动器
    fn select_drive(&mut self, drive: u8) -> Result<(), ()> {
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        unsafe {
            // Bit 4 => DEV
            // Bit 5 => 1
            // Bit 7 => 1
            self.drive_register.write(0xA0 | (drive << 4))
        }
        syskrnl::time::nanowait(400); // Wait at least 400 ns
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        Ok(())
    }

    /// 向ATA设备发送命令的参数
    fn write_command_params(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);
        unsafe {
            self.sector_count_register.write(1);
            self.lba0_register.write(bytes[0]);
            self.lba1_register.write(bytes[1]);
            self.lba2_register.write(bytes[2]);
            self.drive_register.write(bytes[3]);
        }
        Ok(())
    }

    /// 向ATA设备发送命令
    fn write_command(&mut self, cmd: Command) -> Result<(), ()> {
        unsafe { self.command_register.write(cmd as u8) }
        self.wait(400); // Wait at least 400 ns
        self.status(); // Ignore results of first read
        self.clear_interrupt();
        if self.status() == 0 { // Drive does not exist
            return Err(());
        }
        if self.is_error() {
            //debug!("ATA {:?} command errored", cmd);
            //self.debug();
            return Err(());
        }
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, true)?;
        Ok(())
    }

    /// 设置ATA设备的PIO模式
    fn setup_pio(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        self.select_drive(drive)?;
        self.write_command_params(drive, block)?;
        Ok(())
    }

    /// 从ATA设备读取数据
    fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) -> Result<(), ()> {
        debug_assert!(buf.len() == BLOCK_SIZE);
        self.setup_pio(drive, block)?;
        self.write_command(Command::Read)?;
        for chunk in buf.chunks_mut(2) {
            let data = self.read_data().to_le_bytes();
            chunk.clone_from_slice(&data);
        }
        if self.is_error() {
            debugln!("ATA read: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    /// 向ATA设备写入数据
    fn write(&mut self, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
        debug_assert!(buf.len() == BLOCK_SIZE);
        self.setup_pio(drive, block)?;
        self.write_command(Command::Write)?;
        for chunk in buf.chunks(2) {
            let data = u16::from_le_bytes(chunk.try_into().unwrap());
            self.write_data(data);
        }
        if self.is_error() {
            debugln!("ATA write: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    /// 识别ATA设备的种类和信息
    fn identify_drive(&mut self, drive: u8) -> Result<IdentifyResponse, ()> {
        if self.check_floating_bus().is_err() {
            return Ok(IdentifyResponse::None);
        }
        self.select_drive(drive)?;
        self.write_command_params(drive, 0)?;
        if self.write_command(Command::Identify).is_err() {
            return if self.status() == 0 {
                Ok(IdentifyResponse::None)
            } else {
                Err(())
            };
        }
        match (self.lba1(), self.lba2()) {
            (0x00, 0x00) => Ok(IdentifyResponse::Ata([(); 256].map(|_| { self.read_data() }))),
            (0x14, 0xEB) => Ok(IdentifyResponse::Atapi),
            (0x3C, 0xC3) => Ok(IdentifyResponse::Sata),
            (_, _) => Err(()),
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        unsafe {
            self.control_register.write(4); // Set SRST bit
            self.wait(5);                   // Wait at least 5 ns
            self.control_register.write(0); // Then clear it
            self.wait(2000);                // Wait at least 2 ms
        }
    }

    #[allow(dead_code)]
    fn debug(&mut self) {
        unsafe {
            debugln!("ATA status register: 0b{:08b} <BSY|DRDY|#|#|DRQ|#|#|ERR>", self.alternate_status_register.read());
            debugln!("ATA error register:  0b{:08b} <#|#|#|#|#|ABRT|#|#>", self.error_register.read());
        }
    }
}

lazy_static! {
    pub static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

#[derive(Clone)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    blocks: u32,
    model: String,
    serial: String,
}

impl Drive {
    /// 打开一个ATA设备，并返回一个包含ATA设备信息的结构体
    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        let mut buses = BUSES.lock();
        if let Ok(IdentifyResponse::Ata(res)) = buses[bus as usize].identify_drive(dsk) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);
            Some(Self { bus, dsk, model, serial, blocks })
        } else {
            None
        }
    }

    /// 返回ATA设备的块大小
    pub const fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    /// 返回ATA设备的总块数
    pub fn block_count(&self) -> u32 {
        self.blocks
    }

    /// 返回人类可读的大小
    fn humanized_size(&self) -> (usize, String) {
        let size = self.block_size() as usize;
        let count = self.block_count() as usize;
        let bytes = size * count;
        if bytes >> 20 < 1000 {
            (bytes >> 20, String::from("MB"))
        } else {
            (bytes >> 30, String::from("GB"))
        }
    }
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

/// 列出系统中所有可用的ATA设备，并返回一个包含ATA信息的向量
pub fn list() -> Vec<Drive> {
    let mut res = Vec::new();
    for bus in 0..2 {
        for dsk in 0..2 {
            if let Some(drive) = Drive::open(bus, dsk) {
                res.push(drive)
            }
        }
    }
    res
}

/// 从指定的 ATA 设备中读取数据到缓冲区中。
///
/// # 参数
///
/// * `bus` - ATA 总线编号。
/// * `drive` - 驱动器编号。
/// * `block` - 块编号。
/// * `buf` - 缓冲区。
///
/// # 返回值
///
/// 如果读取成功，则返回一个空值。如果读取失败，则返回一个错误值。
pub fn read(bus: u8, drive: u8, block: u32, buf: &mut [u8]) -> Result<(), ()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].read(drive, block, buf)
}

/// 将缓冲区中的数据写入到指定的 ATA 设备中。
///
/// # 参数
///
/// * `bus` - ATA 总线编号。
/// * `drive` - 驱动器编号。
/// * `block` - 块编号。
/// * `buf` - 缓冲区。
///
/// # 返回值
///
/// 如果写入成功，则返回一个空值。如果写入失败，则返回一个错误值。
pub fn write(bus: u8, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buf)
}

pub fn init(){
    let mut buses = BUSES.lock();
    buses.push(Bus::new(0, 0x1F0, 0x3F6, 14));
    buses.push(Bus::new(1, 0x170, 0x376, 15));
    drop(buses);

    for drive in list() {
        debugln!("ATA {}:{} {}\n", drive.bus, drive.dsk, drive);
    }
}



