use alloc::format;

use x86::io::{inl, outl};

use crate::io::qemu::qemu_print;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

pub fn pci_config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let addr: u32 = ((bus as u32) << 16) | ((device as u32) << 11) | ((function as u32) << 8) | ((offset as u32) & 0xFC) | 0x8000_0000u32;
    return unsafe {
        outl(PCI_CONFIG_ADDRESS, addr);
        inl(PCI_CONFIG_DATA)
    };
}

pub fn pci_find_device(device_id: u16, vendor_id: u16) -> (u8, u8, u8) {
    let target = ((device_id as u32) << 16) + vendor_id as u32;
    for bus in 0..=255 {
        for device in 0..32 {
            for function in 0..8 {
                // qemu_print(format!("{},{},{}", bus, device, function).as_str());
                if pci_config_read_u32(bus, device, function, 0) == target {
                    return (bus, device, function);
                }
            }
        }
    }

    // 找不到，找不到
    (0xFF, 0xFF, 0xFF)
}