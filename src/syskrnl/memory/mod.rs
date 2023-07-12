use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use bootloader::BootInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    PhysAddr,
    structures::paging::PageTable,
    VirtAddr,
};
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB};
use crate::{println, syskrnl};


pub mod graphic_support;

pub static mut PHYS_MEM_OFFSET: u64 = 0;
pub static mut MEMORY_MAP: Option<&MemoryMap> = None;
pub static MEMORY_SIZE: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn memory_size() -> u64 {
    MEMORY_SIZE.load(Ordering::Relaxed)
}

pub fn init(bootinfo: &'static BootInfo) {
    interrupts::without_interrupts(|| {
        let mut memory_size = 0;
        for region in bootinfo.memory_map.iter() {
            let start_addr = region.range.start_addr();
            let end_addr = region.range.end_addr();
            memory_size += end_addr - start_addr;
            // println!("MEM [{:#016X}-{:#016X}] {:?}", start_addr, end_addr - 1, region.region_type);
        }
        println!("Memory: {} KB", memory_size >> 10);
        MEMORY_SIZE.store(memory_size, Ordering::Relaxed);

        unsafe { PHYS_MEM_OFFSET = bootinfo.physical_memory_offset };
        unsafe { MEMORY_MAP.replace(&bootinfo.memory_map) };

        let mut mapper = unsafe { mapper(VirtAddr::new(PHYS_MEM_OFFSET)) };
        let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&bootinfo.memory_map) };

        syskrnl::allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

        syskrnl::graphic::enter_wide_mode(&mut mapper, &mut frame_allocator); // 因为需要分配显存，就放在这里了
    });
}

/// 初始化偏移页表
///
/// 这个函数是危险的，因为其调用的函数具有危险性。
/// 详情请见active_level_4_table
pub unsafe fn mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// 返回用于激活Level 4页表的引用。
///
/// 必须指出，这个函数是危险的。如果Physical Memory Offset错误给出，
/// 将会造成panic。此外，重复调用这个函数也是危险的，因为它会返回静态
/// 可变引用。
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let physics = level_4_table_frame.start_address();
    let virt = physical_memory_offset + physics.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// 将给定的虚拟地址转换为映射的物理地址，或者None（如果不存在的话）
///
/// 这个函数是危险的。调用者必须保证完整的物理地址已经被映射到虚拟地址上，
/// 且在Physical Memory Offset所申明的位置上。
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

/// 私有函数
///
/// 虽然没有给出unsafe，但是它是危险的。
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // 从CR3读当前活跃的L4页帧
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // 遍历多级页表
    for &index in &table_indexes {
        // 将帧转换为页表引用
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // 从页表读取位址并更新帧
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Not Supported HugeFrame")
        };
    }

    // 计算物理地址
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// 创建一个映射，将给定的页映射到0xb8000
///
/// FIXME 删了这个函数
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    // 调用者必须保证所请求的帧未被使用
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("Map_to Failed").flush();
}

/// 虚拟的帧分配器
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// 帧分配器，返回BootLoader的内存映射中的可用帧
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
}

impl BootInfoFrameAllocator {
    /// 使用传递的内存映射创建一个帧分配器
    ///
    /// 函数不安全，因为调用者必须保证memory_map的正确性
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
        }
    }

    /// 返回一个可用帧的迭代器
    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
        // 获取内存中的可用区域
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 将这些区域映射到他们的地址范围内
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 转换为帧起始位置的迭代器
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 通过帧起始位置创建PhysFrame类
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let next = ALLOCATED_FRAMES.fetch_add(1, Ordering::SeqCst);
        //debug!("Allocate frame {} / {}", next, self.usable_frames().count());

        // FIXME: creating an iterator for each allocation is very slow if
        // the heap is larger than a few megabytes.
        self.usable_frames().nth(next)
    }
}

