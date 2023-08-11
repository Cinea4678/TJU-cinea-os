use volatile::Volatile;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB};

use crate::debugln;
use crate::syskrnl::io;

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
pub struct HbaMem {     // 0x00 - 0x2B, Generic Host Control
    pub cap: u32,       // 0x00, Host capability
    pub ghc: u32,       // 0x04, Global host control
    pub is: u32,        // 0x08, Interrupt status
    pub pi: u32,        // 0x0C, Port implemented
    pub vs: u32,        // 0x10, Version
    pub ccc_ctl: u32,   // 0x14, Command completion coalescing control
    pub ccc_pts: u32,   // 0x18, Command completion coalescing ports
    pub em_loc: u32,    // 0x1C, Enclosure management location
    pub em_ctl: u32,    // 0x20, Enclosure management control
    pub cap2: u32,      // 0x24, Host capabilities extended
    pub bohc: u32,      // 0x28, BIOS/OS handoff control and status

    // 0x2C - 0x9F, Reserved
    pub rsv: [Volatile<u8>; 0xA0 - 0x2C],

    // 0xA0 - 0xFF, Vendor specific registers
    pub vendor: [Volatile<u8>; 0x100 - 0xA0],

    // 0x100 - 0x10FF, Port control registers
    pub ports: [HbaPort; 1], // 1 ~ 32
}

#[repr(C)]
#[derive(Debug)]
pub struct HbaPort {
    pub clb: u32,       // 0x00, command list base address, 1K-byte aligned
    pub clbu: u32,      // 0x04, command list base address upper 32 bits
    pub fb: u32,        // 0x08, FIS base address, 256-byte aligned
    pub fbu: u32,       // 0x0C, FIS base address upper 32 bits
    pub is: u32,        // 0x10, interrupt status
    pub ie: u32,        // 0x14, interrupt enable
    pub cmd: u32,       // 0x18, command and status
    pub rsv0: u32,      // 0x1C, Reserved
    pub tfd: u32,       // 0x20, task file data
    pub sig: u32,       // 0x24, signature
    pub ssts: u32,      // 0x28, SATA status (SCR0:SStatus)
    pub sctl: u32,      // 0x2C, SATA control (SCR2:SControl)
    pub serr: u32,      // 0x30, SATA error (SCR1:SError)
    pub sact: u32,      // 0x34, SATA active (SCR3:SActive)
    pub ci: u32,        // 0x38, command issue
    pub sntf: u32,      // 0x3C, SATA notification (SCR4:SNotification)
    pub fbs: u32,       // 0x40, FIS-based switch control
    pub rsv1: [u32; 11],  // 0x44 ~ 0x6F, Reserved
    pub vendor: [u32; 4], // 0x70 ~ 0x7F, vendor specific
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
    pub ctba: u32,   // Command table descriptor base address
    pub ctbau: u32,  // Command table descriptor base address upper 32 bits

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

const AHCI_BASE: u32 = 0x400000;    // 4M

const HBA_PxCMD_ST: u32 = 0x0001;
const HBA_PxCMD_FRE: u32 = 0x0010;
const HBA_PxCMD_FR: u32 = 0x4000;
const HBA_PxCMD_CR: u32 = 0x8000;

fn port_rebase(port: &mut HbaPort, port_no: u32) {
    // 停止命令引擎
    stop_cmd(port);

    // 命令列表偏移量：1K * port_no
    // 命令列表条目大小：32
    // 命令列表条目最大数量：32
    // 命令列表最大大小：32 * 32 = 1K per Port （看懂了）
    port.clb.write(AHCI_BASE + (port_no << 10));
    port.clbu.write(0);
    // TODO: MEMSET

    // FIS 偏移量：32K + 256 * port_no
    // FIS 条目大小：256byte per Port
    port.fb.write(AHCI_BASE + (32 << 10) + (port_no << 8));
    port.fbu.write(0);
    // TODO: MEMSET

    // 命令表偏移量：40K + 8K * port_no
    // 命令表大小：256 * 32 = 8K per Port
    let cmd_header = port.clb.read();
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

pub fn init() {
    let mut abar = unsafe { &*(ABAR_ADDR as *mut HbaMem) };
    probe_port(abar);
}
