use alloc::boxed::Box;
use alloc::slice;
use core::{mem, ptr};
use core::alloc::Layout;

use volatile::Volatile;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB};

use crate::debugln;
use crate::syskrnl::io;
use crate::syskrnl::memory::translate_addr;

#[repr(u8)]
pub enum FisType{
    FisTypeRegH2d = 0x27,	// Register FIS - host to device
    FisTypeRegD2h = 0x34,	// Register FIS - device to host
    FisTypeDmaAct = 0x39,	// DMA activate FIS - device to host
    FisTypeDmaSetup = 0x41,	// DMA setup FIS - bidirectional
    FisTypeData = 0x46,	// Data FIS - bidirectional
    FisTypeBist = 0x58,	// BIST activate FIS - bidirectional
    FisTypePioSetup = 0x5F,	// PIO setup FIS - device to host
    FisTypeDevBits = 0xA1,	// Set device bits FIS - device to host
}

#[repr(C)]
#[derive(Debug)]
pub struct FisRegH2d {
    // DWORD 0
    fis_type: u8,      // FIS_TYPE_REG_H2D

    // This is a byte with mixed fields, so I will use a private u8 and provide getters and setters for its parts.
    _pmport_rsv0_c: u8,

    command: u8,       // Command register
    featurel: u8,      // Feature register, 7:0

    // DWORD 1
    lba0: u8,          // LBA low register, 7:0
    lba1: u8,          // LBA mid register, 15:8
    lba2: u8,          // LBA high register, 23:16
    device: u8,        // Device register

    // DWORD 2
    lba3: u8,          // LBA register, 31:24
    lba4: u8,          // LBA register, 39:32
    lba5: u8,          // LBA register, 47:40
    featureh: u8,      // Feature register, 15:8

    // DWORD 3
    countl: u8,        // Count register, 7:0
    counth: u8,        // Count register, 15:8
    icc: u8,           // Isochronous command completion
    control: u8,       // Control register

    // DWORD 4
    rsv1: [u8; 4],     // Reserved
}

impl FisRegH2d {
    // Getter and Setter for pmport (4 bits)
    pub fn pmport(&self) -> u8 {
        self._pmport_rsv0_c & 0x0F
    }

    pub fn set_pmport(&mut self, value: u8) {
        self._pmport_rsv0_c = (self._pmport_rsv0_c & 0xF0) | (value & 0x0F);
    }

    // Getter and Setter for rsv0 (3 bits)
    pub fn rsv0(&self) -> u8 {
        (self._pmport_rsv0_c >> 4) & 0x07
    }

    pub fn set_rsv0(&mut self, value: u8) {
        self._pmport_rsv0_c = (self._pmport_rsv0_c & 0x8F) | ((value & 0x07) << 4);
    }

    // Getter and Setter for c (1 bit)
    pub fn c(&self) -> u8 {
        (self._pmport_rsv0_c >> 7) & 0x01
    }

    pub fn set_c(&mut self, value: u8) {
        self._pmport_rsv0_c = (self._pmport_rsv0_c & 0x7F) | ((value & 0x01) << 7);
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct HbaMem {     // 0x00 - 0x2B, Generic Host Control
    pub cap: Volatile<u32>,       // 0x00, Host capability
    pub ghc: Volatile<u32>,       // 0x04, Global host control
    pub is: Volatile<u32>,        // 0x08, Interrupt status
    pub pi: Volatile<u32>,        // 0x0C, Port implemented
    pub vs: Volatile<u32>,        // 0x10, Version
    pub ccc_ctl: Volatile<u32>,   // 0x14, Command completion coalescing control
    pub ccc_pts: Volatile<u32>,   // 0x18, Command completion coalescing ports
    pub em_loc: Volatile<u32>,    // 0x1C, Enclosure management location
    pub em_ctl: Volatile<u32>,    // 0x20, Enclosure management control
    pub cap2: Volatile<u32>,      // 0x24, Host capabilities extended
    pub bohc: Volatile<u32>,      // 0x28, BIOS/OS handoff control and status

    // 0x2C - 0x9F, Reserved
    pub rsv: [Volatile<u8>; 0xA0 - 0x2C],

    // 0xA0 - 0xFF, Vendor specific registers
    pub vendor: [Volatile<u8>; 0x100 - 0xA0],

    // 0x100 - 0x10FF, Port control registers
    pub ports: [HbaPort; 32], // 1 ~ 32
}

#[repr(C)]
#[derive(Debug)]
pub struct HbaPort {
    pub clb: Volatile<u64>,       // 0x00, command list base address, 1K-byte aligned
    pub fb: Volatile<u64>,        // 0x08, FIS base address, 256-byte aligned
    pub is: Volatile<u32>,        // 0x10, interrupt status
    pub ie: Volatile<u32>,        // 0x14, interrupt enable
    pub cmd: Volatile<u32>,       // 0x18, command and status
    pub rsv0: Volatile<u32>,      // 0x1C, Reserved
    pub tfd: Volatile<u32>,       // 0x20, task file data
    pub sig: Volatile<u32>,       // 0x24, signature
    pub ssts: Volatile<u32>,      // 0x28, SATA status (SCR0:SStatus)
    pub sctl: Volatile<u32>,      // 0x2C, SATA control (SCR2:SControl)
    pub serr: Volatile<u32>,      // 0x30, SATA error (SCR1:SError)
    pub sact: Volatile<u32>,      // 0x34, SATA active (SCR3:SActive)
    pub ci: Volatile<u32>,        // 0x38, command issue
    pub sntf: Volatile<u32>,      // 0x3C, SATA notification (SCR4:SNotification)
    pub fbs: Volatile<u32>,       // 0x40, FIS-based switch control
    pub rsv1: [Volatile<u32>; 11],  // 0x44 ~ 0x6F, Reserved
    pub vendor: [Volatile<u32>; 4], // 0x70 ~ 0x7F, vendor specific
}

#[repr(C)]
#[derive(Debug)]
pub struct HbaCmdHeader{
    // DW0
    dw0: u16,      // Combines cfl, a, w, p, r, b, c, rsv0, and pmp into two bytes
    pub prdtl: u16,    // Physical region descriptor table length in entries

    // DW1
    pub prdbc: Volatile<u32>,  // Physical region descriptor byte count transferred

    // DW2, 3
    pub ctba: u64,   // Command table descriptor base address

    // DW4 - 7
    pub rsv1: [u32; 4],  // Reserved
}

impl HbaCmdHeader {
    /// CFL: Command FIS length in DWORDS, 2 ~ 16
    pub fn cfl(&self) -> u8 {
        (self.dw0 & 0b0001_1111) as u8
    }

    pub fn set_cfl(&mut self, value: u8) {
        self.dw0 = (self.dw0 & !0b0001_1111) | ((value as u16) & 0b0001_1111);
    }

    /// A: ATAPI
    pub fn a(&self) -> bool {
        (self.dw0 & 0b0010_0000) != 0
    }

    pub fn set_a(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b0010_0000;
        } else {
            self.dw0 &= !0b0010_0000;
        }
    }

    /// W: Write, 1: H2D, 0: D2H
    pub fn w(&self) -> bool {
        (self.dw0 & 0b0100_0000) != 0
    }

    pub fn set_w(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b0100_0000;
        } else {
            self.dw0 &= !0b0100_0000;
        }
    }

    // P: Prefetchable
    pub fn p(&self) -> bool {
        (self.dw0 & 0b1000_0000) != 0
    }

    pub fn set_p(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b1000_0000;
        } else {
            self.dw0 &= !0b1000_0000;
        }
    }

    // R: Reset
    pub fn r(&self) -> bool {
        (self.dw0 & 0b1_0000_0000) != 0
    }

    pub fn set_r(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b1_0000_0000;
        } else {
            self.dw0 &= !0b1_0000_0000;
        }
    }

    // B: BIST
    pub fn b(&self) -> bool {
        (self.dw0 & 0b10_0000_0000) != 0
    }

    pub fn set_b(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b10_0000_0000;
        } else {
            self.dw0 &= !0b10_0000_0000;
        }
    }

    // C: Clear busy upon R_OK
    pub fn c(&self) -> bool {
        (self.dw0 & 0b100_0000_0000) != 0
    }

    pub fn set_c(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b100_0000_0000;
        } else {
            self.dw0 &= !0b100_0000_0000;
        }
    }

    // RSV0: Reserved
    pub fn rsv0(&self) -> bool {
        (self.dw0 & 0b1000_0000_0000) != 0
    }

    pub fn set_rsv0(&mut self, value: bool) {
        if value {
            self.dw0 |= 0b1000_0000_0000;
        } else {
            self.dw0 &= !0b1000_0000_0000;
        }
    }

    // PMP: Port multiplier port
    pub fn pmp(&self) -> u8 {
        ((self.dw0 >> 12) & 0b1111) as u8
    }

    pub fn set_pmp(&mut self, value: u8) {
        self.dw0 = (self.dw0 & !(0b1111 << 12)) | (((value as u16) & 0b1111) << 12);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HbaCmdTbl {
    // 0x00
    pub cfis: [u8; 64],    // Command FIS

    // 0x40
    pub acmd: [u8; 16],    // ATAPI command, 12 or 16 byte

    // 0x50
    pub rsv: [u8; 48],    // Reserved

    // 0x80
    prdt_entry: [HbaPrdtEntry; 0],    // Physical region descriptor table entries, 0 ~ 65535
}

impl HbaCmdTbl {
    pub fn with_entries(n: usize) -> Box<Self> {
        let layout = Layout::from_size_align(
            mem::size_of::<Self>() + n * mem::size_of::<HbaPrdtEntry>(),
            mem::align_of::<HbaPrdtEntry>(),
        ).unwrap();

        unsafe {
            let raw = alloc::alloc::alloc_zeroed(layout);
            let hba = &mut *(raw as *mut HbaCmdTbl);

            Box::from_raw(hba)
        }
    }

    pub fn pred_entries(&mut self, len: usize) -> &[HbaPrdtEntry] {
        unsafe { slice::from_raw_parts(self.prdt_entry.as_ptr(), len) }
    }

    pub fn prdt_entries_mut(&mut self, len: usize) -> &mut [HbaPrdtEntry] {
        unsafe { slice::from_raw_parts_mut(self.prdt_entry.as_mut_ptr(), len) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HbaPrdtEntry {
    pub dba: u64,
    // Data base address
    pub rsv0: u32,      // Reserved

    // DW3
    pub dw3: u32,       // Byte count/Interrupt on completion
}

impl HbaPrdtEntry {
    /// Byte count, 4M max
    pub fn dbc(&self) -> u32 {
        self.dw3 & 0b00111111_11111111_11111111
    }

    pub fn set_dbc(&mut self, value: u32) {
        self.dw3 = (self.dw3 & !0b00111111_11111111_11111111) | (value & 0b00111111_11111111_11111111)
    }

    /// Interrupt on completion
    pub fn i(&self) -> bool {
        (self.dw3 >> 31) > 0
    }

    pub fn set_i(&mut self, value: bool) {
        if value {
            self.dw3 |= 1 << 31
        } else {
            self.dw3 &= !(1 << 31)
        }
    }
}

const SATA_SIG_ATA: u32 = 0x0000_0101;      // SATA drive
const SATA_SIG_ATAPI: u32 = 0xEB14_0101;    // SATAPI drive
const SATA_SIG_SEMB: u32 = 0xC33C_0101;     // Enclosure management bridge
const SATA_SIG_PM: u32 = 0x9669_0101;       // Port multiplier

const AHCI_DEV_NULL: u8 = 0;
const AHCI_DEV_SATA: u8 = 1;
const AHCI_DEV_SEMB: u8 = 2;
const AHCI_DEV_PM: u8 = 3;
const AHCI_DEV_SATAPI: u8 = 4;

const HBA_PORT_IPM_ACTIVE: u32 = 1;
const HBA_PORT_DET_PRESENT: u32 = 3;

fn probe_port(abar: &HbaMem) {
    // 在已实现的端口中扫描磁盘
    let mut pi = abar.pi.read();
    let mut i = 0;
    while i < 32 {
        if pi & 1 > 0 {
            let dt = check_type(&abar.ports[i]);
            match dt {
                AHCI_DEV_SATA => {
                    debugln!("SATA drive found at port {}", i);
                }
                AHCI_DEV_SATAPI => {
                    debugln!("SATAPI drive found at port {}", i);
                }
                AHCI_DEV_SEMB => {
                    debugln!("SEMB drive found at port {}", i);
                }
                AHCI_DEV_PM => {
                    debugln!("PM drive found at port {}", i);
                }
                _ => {
                    debugln!("No drive found at port {}", i);
                }
            }
        }
        pi >>= 1;
        i += 1;
    }
}

/// 检查设备类型
fn check_type(port: &HbaPort) -> u8 {
    let ssts = port.ssts.read();

    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;

    if det != HBA_PORT_DET_PRESENT || ipm != HBA_PORT_IPM_ACTIVE {
        return AHCI_DEV_NULL;
    }

    match port.sig.read() {
        SATA_SIG_ATAPI => AHCI_DEV_SATAPI,
        SATA_SIG_SEMB => AHCI_DEV_SEMB,
        SATA_SIG_PM => AHCI_DEV_PM,
        _ => AHCI_DEV_SATA
    }
}

const AHCI_BASE: u64 = 0xffffffff_00400000;
const AHCI_PHYSIC_BASE: u64 = 0x00000000_00400000;    // 4M

#[allow(non_upper_case_globals)]
const HBA_PxCMD_ST: u32 = 0x0001;

#[allow(non_upper_case_globals)]
const HBA_PxCMD_FRE: u32 = 0x0010;

#[allow(non_upper_case_globals)]
const HBA_PxCMD_FR: u32 = 0x4000;

#[allow(non_upper_case_globals)]
const HBA_PxCMD_CR: u32 = 0x8000;

fn port_rebase(port: &mut HbaPort, port_no: u64) {
    // 停止命令引擎
    stop_cmd(port);

    // 命令列表偏移量：1K * port_no
    // 命令列表条目大小：32
    // 命令列表条目最大数量：32
    // 命令列表最大大小：32 * 32 = 1K per Port （看懂了）
    port.clb.write(AHCI_PHYSIC_BASE + (port_no << 10));
    unsafe { ptr::write_bytes((port.clb.read() - AHCI_PHYSIC_BASE + AHCI_BASE) as *mut u8, 0, 1024); }

    // FIS 偏移量：32K + 256 * port_no
    // FIS 条目大小：256byte per Port
    port.fb.write(AHCI_PHYSIC_BASE + (32 << 10) + (port_no << 8));
    unsafe { ptr::write_bytes((port.fb.read() - AHCI_PHYSIC_BASE + AHCI_BASE) as *mut u8, 0, 256); }

    // 命令表偏移量：40K + 8K * port_no
    // 命令表大小：256 * 32 = 8K per Port
    let cmd_header = unsafe { slice::from_raw_parts_mut((port.clb.read() - AHCI_PHYSIC_BASE + AHCI_BASE) as *mut HbaCmdHeader, 32) };
    for i in 0..32 {
        cmd_header[i].prdtl = 8;    // 8 prdt entries per command table; 256 bytes per command table, 64+16+48+16*8


        // 命令表偏移量：40K + 8K * port_no +  cmd_header_index * 256
        cmd_header[i].ctba = AHCI_PHYSIC_BASE + (40 << 10) + (port_no << 13) + ((i as u64) << 8);
        unsafe { ptr::write_bytes((cmd_header[i].ctba - AHCI_PHYSIC_BASE + AHCI_BASE) as *mut u8, 0, 256); }
    }

    start_cmd(port); // 重启命令引擎
}

/// 启动命令引擎
fn start_cmd(port: &mut HbaPort) {
    while port.cmd.read() & HBA_PxCMD_CR > 0 {}  // 等待CR位被清除

    // 置FRE和ST位
    port.cmd.update(|x| *x |= HBA_PxCMD_FRE);
    port.cmd.update(|x| *x |= HBA_PxCMD_ST);
}

/// 停止命令引擎
fn stop_cmd(port: &mut HbaPort) {
    // 清除ST
    port.cmd.update(|x| *x &= !HBA_PxCMD_ST);
    // 清除FRE
    port.cmd.update(|x| *x &= !HBA_PxCMD_FRE);

    // 等待FR和CR位被清除
    while port.cmd.read() & HBA_PxCMD_FR > 0 || port.cmd.read() & HBA_PxCMD_CR > 0 {}
}

const ABAR_ADDR: u64 = 0xffffffff_febf1000;

pub fn create_abar_memory_mapping(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let controller = io::pci::pci_find_device_by_class_code(0x01, 0x06);
    let abar_addr = io::pci::pci_config_read_u32(controller.0, controller.1, controller.2, 0x24);

    use x86_64::structures::paging::PageTableFlags as Flags;

    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(ABAR_ADDR));
    let frame = PhysFrame::containing_address(PhysAddr::new(abar_addr as u64));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("Map_to_AHCI aBar Memory Failed").flush();
}

pub fn create_ahci_memory_mapping(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // We need 16 pages
    for i in 0..16 {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(AHCI_BASE + 0x1000 * i));
        let frame = PhysFrame::containing_address(PhysAddr::new(AHCI_PHYSIC_BASE + 0x1000 * i));
        let flags = Flags::PRESENT | Flags::WRITABLE;

        let map_to_result = unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)
        };
        map_to_result.expect("Map_to_AHCI Memory Failed").flush();
    }
}

const HBA_GHC_HR: u32 = 0x0001;
// HBA Reset
const HBA_GHC_AE: u32 = 0x8000; // AHCI Enable

fn start_hba(abar: &mut HbaMem) {
    abar.ghc.update(|x| *x |= HBA_GHC_AE);
    abar.ghc.update(|x| *x |= HBA_GHC_HR);
    while (abar.ghc.read() & HBA_GHC_HR) != 0 {}
    abar.ghc.update(|x| *x |= HBA_GHC_AE);
}

pub fn init() {
    let abar = unsafe { &mut *(ABAR_ADDR as *mut HbaMem) };

    probe_port(abar);

    start_hba(abar);

    port_rebase(&mut abar.ports[0], 0);

    debugln!("成功完成AHCI内存区域初始化\n\n开始读盘测试：端口0，扇区0，长度一扇区：");

    let mut buf = [0u8; 512];
    match abar.ports[0].read(0, 1, &mut buf) {
        false => { debugln!("读盘失败QAQ") },
        true => {
            debugln!("读盘成功，请核对：{:#?}", buf);
        }
    }
}

const ATA_CMD_READ_DMA_EX: u8 = 0x25;

const ATA_DEV_BUSY: u8 = 0x80;
const ATA_DEV_DRQ: u8 = 0x08;

#[allow(non_upper_case_globals)]
const HBA_PxIS_TFES: u32 = 1 << 30;

impl HbaPort {
    fn find_cmdslot(&self) -> Option<u32> {
        let mut slots = self.sact.read() | self.ci.read();
        for i in 0..32 {
            if (slots & 1) == 0 {
                return Some(i);
            }
            slots >>= 1;
        }
        debugln!("Cannot find free command list entry");
        None
    }

    pub fn read(&mut self, pos: u64, count: u32, buf: &mut [u8]) -> bool {
        self.is.write((-1i32) as u32);
        let mut spin = 0;
        if let Some(slot) = self.find_cmdslot() {
            let cmd_header = unsafe { &mut *((self.clb.read() - AHCI_PHYSIC_BASE + AHCI_BASE) as *mut HbaCmdHeader).add(slot as usize) };
            cmd_header.set_cfl((mem::size_of::<FisRegH2d>() / mem::size_of::<u32>()) as u8); // Command FIS size
            cmd_header.set_w(false);        // Read from device
            cmd_header.prdtl = (((count - 1) >> 4) + 1) as u16;

            let mut cmd_table = HbaCmdTbl::with_entries(cmd_header.prdtl as usize);

            let mut buf_addr = unsafe { translate_addr(buf.as_mut_ptr() as u64).unwrap() };
            let mut count = count;
            let prdt_entry = cmd_table.prdt_entries_mut(cmd_header.prdtl as usize);

            for i in 0..cmd_header.prdtl as usize {
                if i < cmd_header.prdtl as usize - 1 {
                    prdt_entry[i].dba = buf_addr;
                    prdt_entry[i].set_dbc(8 * 1024 - 1); // 8K bytes (this value should always be set to 1 less than the actual value)
                    prdt_entry[i].set_i(true);
                    buf_addr += 8 * 1024; // 8K bytes
                    count -= 16;          // 16 sectors
                } else {
                    prdt_entry[i].dba = buf_addr;
                    prdt_entry[i].set_dbc((count << 9) - 1);  // 512 bytes per sector
                    prdt_entry[i].set_i(true);
                }
            }

            let cmd_fis = unsafe { &mut *(cmd_table.cfis.as_ptr() as *mut FisRegH2d) };
            cmd_fis.fis_type = FisType::FisTypeRegH2d as u8;
            cmd_fis.set_c(1); // Command
            cmd_fis.command = ATA_CMD_READ_DMA_EX;

            cmd_fis.lba0 = pos as u8;
            cmd_fis.lba1 = (pos >> 8) as u8;
            cmd_fis.lba2 = (pos >> 16) as u8;
            cmd_fis.lba3 = (pos >> 24) as u8;
            cmd_fis.lba4 = (pos >> 32) as u8;
            cmd_fis.lba5 = (pos >> 40) as u8;

            cmd_fis.device = 1 << 6;  // LBA mode
            cmd_fis.countl = (count & 0xFF) as u8;
            cmd_fis.counth = ((count >> 8) & 0xFF) as u8;

            cmd_header.ctba = unsafe { translate_addr(Box::into_raw(cmd_table) as u64).unwrap() };

            // The below loop waits until the port is no longer busy before issuing a new command
            while (self.tfd.read() & (ATA_DEV_BUSY | ATA_DEV_DRQ) as u32) > 0 && spin < 1000000 {
                spin += 1;
            }
            if spin == 1000000 {
                debugln!("Port is hung");
                false
            } else {
                self.ci.update(|x| *x |= 1 << slot);  // 签发命令

                debugln!("开始等待");
                // Wait for completion
                loop {
                    // In some longer duration reads, it may be helpful to spin on the DPS bit
                    // in the PxIS port field as well (1 << 5)
                    if self.ci.read() & (1 << slot) == 0 {
                        break;
                    }
                    if self.is.read() & HBA_PxIS_TFES > 0 {
                        debugln!("Read disk error");
                        return false;
                    }
                }

                // Check again
                if self.is.read() & HBA_PxIS_TFES > 0 {
                    debugln!("Read disk error");
                    false
                } else {
                    true
                }
            }
        } else {
            false
        }
    }
}


