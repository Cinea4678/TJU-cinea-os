use crate::debugln;

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
    pub rsv: [u8; 0xA0 - 0x2C],

    // 0xA0 - 0xFF, Vendor specific registers
    pub vendor: [u8; 0x100 - 0xA0],

    // 0x100 - 0x10FF, Port control registers
    pub ports: [HbaPort; 1], // 1 ~ 32
}

#[repr(C)]
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
    let mut pi = abar.pi;
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
    let ssts = port.ssts;

    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;

    if det != HBA_PORT_DET_PRESENT || ipm != HBA_PORT_IPM_ACTIVE {
        return AHCI_DEV_NULL;
    }

    match port.sig {
        SATA_SIG_ATAPI => AHCI_DEV_SATAPI,
        SATA_SIG_SEMB => AHCI_DEV_SEMB,
        SATA_SIG_PM => AHCI_DEV_PM,
        _ => AHCI_DEV_SATA
    }
}
