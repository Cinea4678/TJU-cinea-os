use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

// 配置区域
const NEEDED_PAGE_NUM: usize = 469;
pub const START_VIRT_ADDR: u64 = 0xC000_0000;

/// 初始化显存
pub fn create_graphic_memory_mapping(mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>, start_physic_addr: u64) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    for i in 0..NEEDED_PAGE_NUM {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(START_VIRT_ADDR + 0x1000 * i as u64));
        let frame = PhysFrame::containing_address(PhysAddr::new(start_physic_addr + 0x1000 * i as u64));
        let flags = Flags::PRESENT | Flags::WRITABLE;

        let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
        map_to_result.expect("Map_to_GraphicMemory Failed").flush();
    }
}
