use alloc::format;

use x86::io::outw;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, Size4KiB};
use crate::memory::graphic_support::create_graphic_memory_mapping;

use crate::pci::{pci_config_read_u32, pci_find_device};
use crate::qemu::qemu_print;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA: u16 = 0x01CF;

#[allow(dead_code)]
#[repr(u16)]
enum VbeDispiIndex {
    Id = 0,
    Xres,
    Yres,
    Bpp,
    Enable,
    Bank,
    VirtWidth,
    VirtHeight,
    XOffset,
    YOffset,
}

#[allow(dead_code)]
#[repr(u16)]
enum VbeDispiBpp {
    _4 = 4,
    _8 = 8,
    _24 = 24,
    _32 = 32,
    // 省略了很多我不可能用得到的深度
}

unsafe fn bga_write_register(index: u16, value: u16) {
    outw(VBE_DISPI_IOPORT_INDEX, index);
    outw(VBE_DISPI_IOPORT_DATA, value);
}

pub unsafe fn bga_enter_wide(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) {
    // 禁用VBE
    bga_write_register(VbeDispiIndex::Enable as u16, 0);

    // 设置显示模式
    bga_write_register(VbeDispiIndex::Xres as u16, super::WIDTH as u16);
    bga_write_register(VbeDispiIndex::Yres as u16, super::HEIGHT as u16);
    bga_write_register(VbeDispiIndex::Bpp as u16, VbeDispiBpp::_32 as u16);

    // 启用VBE
    bga_write_register(VbeDispiIndex::Enable as u16, 0x41);

    // 获取LFB地址
    let device = pci_find_device(0x1111, 0x1234);
    qemu_print(format!("LFB device is {:?}\n", device).as_str());
    let address = pci_config_read_u32(device.0, device.1, device.2, 0x10);
    qemu_print(format!("We get LFB address:{:?}\n", address).as_str());

    // 初始化显存
    create_graphic_memory_mapping(mapper, frame_allocator, address as u64);
}