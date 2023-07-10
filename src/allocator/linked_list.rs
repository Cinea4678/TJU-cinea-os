/// 書：
///
/// 本文件由phil-opp.com的版本修改而来。
/// 改进了空闲区块的排序原则，并增加了dealloc时碎片区块的合并。
///

use core::alloc::{GlobalAlloc, Layout};
use core::mem;

use crate::allocator::{align_up, Locked};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode {
            size,
            next: None,
        }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// 新建一个Bump Allocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// 根据给定堆区间范围初始化Bump Allocator
    ///
    /// 很显然，这个方法是不安全的，因为给定的区间需要确保未被使用，此外这个函数也不能被多次调用
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// 将指定的内存区域增加到链表中
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // 确保这个空闲区域和链表是适配的
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size>= mem::size_of::<ListNode>());

        // 先寻找左右两边是否也为空闲

        let (mut left_found, mut right_found) = (false, false);
        let (mut addr,mut size) = (addr,size);
        let mut current = &mut self.head;
        while let Some(ref mut region)=current.next {
            if !left_found && region.end_addr() == addr{
                left_found = true;
                addr = region.start_addr();
                size += region.size;
                current.next = region.next.take();
            }else if !right_found && region.start_addr() == addr+size {
                right_found = true;
                size += region.size;
                current.next = region.next.take();
            }else if left_found && right_found {
                break;
            }else {
                current = current.next.as_mut().unwrap();
            }
        }

        // 创建一个新的链表节点，并把它加入到合适位置（按照地址从大到小排序）
        let mut node = ListNode::new(size);
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if region.size <= node.size {
                break;
            }
            current = current.next.as_mut().unwrap();
        }
        node.next = current.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        current.next = Some(&mut *node_ptr);
    }

    /// 寻找一个满足给定大小的空闲区域，并把它从链表中移除
    ///
    /// 返回列表节点和可用区域的起始地址
    fn find_region(&mut self, size: usize, align: usize)
        -> Option<(&'static mut ListNode, usize)>
    {
        let mut current = &mut self.head;
        // 寻找足够的空间
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // 找到了，返回
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // 这个区域不够大，继续往下找
                current = current.next.as_mut().unwrap();
            }
        }

        // 找不到啊找不到，因为你找不到
        None
    }

    /// 尝试使用给定区域进行具有给定大小和对齐方式的分配
    ///
    /// 成功时返回分配起始地址
    fn alloc_from_region(region: &ListNode, size: usize, align: usize)
                         -> Result<usize, ()>{
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr(){
            // 这个区域不够大
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // 剩余空间不足以容纳一个ListNode
            return Err(());
        }

        Ok(alloc_start)
    }

    /// 调整给出的布局，使得其内存区域也能满足存储一个链表节点的需求
    ///
    /// 返回调整后的布局大小和对齐方式
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout:Layout)->*mut u8{
        // 进行布局调整
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region,alloc_start)) = allocator.find_region(size,align) {
            // 找到了，进行分配
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                // 有剩余空间，把它加入到链表中
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            // 没找到，返回空指针
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size,_) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size);
    }
}
