use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use lazy_static::lazy_static;
use object::{Object, ObjectSegment};
use spin::{Mutex, RwLock};
use x86_64::registers::control::Cr3;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame};
use x86_64::VirtAddr;

use cinea_os_sysapi::ExitCode;

use crate::{debugln, syskrnl};
use crate::syskrnl::allocator::{alloc_pages, fix_page_fault_in_userspace, Locked};
use crate::syskrnl::allocator::linked_list::LinkedListAllocator;
use crate::syskrnl::fs::OpenFileHandle;
use crate::syskrnl::schedule::ProcessScheduler;
use crate::syskrnl::schedule::roundroll::RoundRollScheduler;

// const MAX_FILE_HANDLES: usize = 64;
/// 最大进程数，先写2个，后面再改
const MAX_PROCS: usize = 4;
const MAX_PROC_SIZE: usize = 10 << 20;
#[allow(dead_code)]
const MAX_FILE_HANDLES: usize = 64;

pub static PID: AtomicUsize = AtomicUsize::new(0);
pub static MAX_PID: AtomicUsize = AtomicUsize::new(1);

pub static PROC_HEAP_ADDR: AtomicUsize = AtomicUsize::new(0x0002_0000_0000);
const DEFAULT_HEAP_SIZE: usize = 0x4000; // 默认堆内存大小

lazy_static! {
    pub static ref SCHEDULER: Mutex<Box<dyn ProcessScheduler + 'static + Send>> = {
        Mutex::new(Box::new(RoundRollScheduler::new()))
    };

    pub static ref PROCESS_TABLE: RwLock<[Box<Process>; MAX_PROCS]> = {
        let table: [Box<Process>; MAX_PROCS] = [(); MAX_PROCS].map(|_| Box::new(Process::new(0)));
        RwLock::new(table)
    };
}

#[derive(Clone, Debug)]
pub struct ProcessData {
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    file_handles: Arc<Mutex<BTreeMap<usize, OpenFileHandle>>>,
}

#[repr(align(8), C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

#[derive(Clone, Debug)]
pub struct Process {
    pub id: usize,
    code_addr: u64,
    stack_addr: u64,
    entry_point: u64,
    page_table_frame: PhysFrame,
    stack_frame: InterruptStackFrameValue,
    registers: Registers,
    data: ProcessData,
    parent: usize,
    allocator: Arc<Locked<LinkedListAllocator>>,
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        let file_handles = Arc::new(Mutex::new(BTreeMap::new()));
        let lock = file_handles.clone();
        let mut lock = lock.lock();
        lock.insert(0, OpenFileHandle {
            id: 0,
            path: "/dev/stdout".to_string(),
            write: true,
            device: true,
        });
        // let mut file_handles = [(); MAX_FILE_HANDLES].map(|_| None);
        // file_handles[0] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdin
        // file_handles[1] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdout
        // file_handles[2] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stderr
        // file_handles[3] = Some(Box::new(Resource::Device(Device::Null))); // stdnull
        Self { env, dir, user, file_handles }
    }
}

impl Process {
    pub fn new(id: usize) -> Self {
        let isf = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new(0),
            stack_segment: 0,
        };
        Self {
            id,
            code_addr: 0,
            stack_addr: 0,
            entry_point: 0,
            stack_frame: isf,
            page_table_frame: Cr3::read().0,
            registers: Registers::default(),
            data: ProcessData::new("/", None),
            parent: 0,
            allocator: Arc::new(Locked::new(LinkedListAllocator::new())),
        }
    }
}

/// 获取当前进程PID
pub fn id() -> usize {
    PID.load(Ordering::SeqCst)
}

/// 设置当前进程PID
pub fn set_id(id: usize) {
    PID.store(id, Ordering::SeqCst);
}

/// 获取当前进程的环境变量
pub fn env(key: &str) -> Option<String> {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.data.env.get(key).cloned()
}

/// 获取当前进程的环境变量
pub fn envs() -> BTreeMap<String, String> {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.data.env.clone()
}

/// 获取当前进程的工作目录
pub fn dir() -> String {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.data.dir.clone()
}

/// 获取当前进程的用户名
pub fn user() -> Option<String> {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.data.user.clone()
}

/// 设置当前进程的环境变量
pub fn set_env(key: &str, val: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.env.insert(key.into(), val.into());
}

/// 设置当前进程的工作目录
pub fn set_dir(dir: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.dir = dir.into();
}

/// 设置当前进程的用户名
pub fn set_user(user: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.user = Some(user.into())
}

/// 获取当前进程的代码地址
pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.code_addr
}

/// 设置当前进程的代码地址
pub fn set_code_addr(addr: u64) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.code_addr = addr;
}

/// 偏移地址转换实际地址
pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    let base = code_addr();
    if addr < base {
        (base + addr) as *mut u8
    } else {
        addr as *mut u8
    }
}

/// 获取当前进程的寄存器
pub fn registers() -> Registers {
    let table = PROCESS_TABLE.read();
    let process = &table[id()];
    process.registers
}

/// 设置当前进程的寄存器
pub fn set_registers(regs: Registers) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.registers = regs
}

/// 获取当前进程的栈帧
pub fn stack_frame() -> InterruptStackFrameValue {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.stack_frame
}

/// 设置当前进程的栈帧
pub fn set_stack_frame(stack_frame: InterruptStackFrameValue) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.stack_frame = stack_frame;
}

pub unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.page_table_frame
}

pub unsafe fn set_page_table_frame(frame: PhysFrame) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.page_table_frame = frame
}

/// 获取当前进程的堆分配器
pub fn heap_allocator() -> Arc<Locked<LinkedListAllocator>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.allocator.clone()
}

/// 生长当前进程的堆
pub fn allocator_grow(size: usize) {
    let page_table = unsafe { page_table() };
    let phys_mem_offset = unsafe { syskrnl::memory::PHYS_MEM_OFFSET };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };

    let table = PROCESS_TABLE.write();
    let allocator = table[id()].allocator.clone();
    let addr = PROC_HEAP_ADDR.fetch_add(size, Ordering::SeqCst);
    alloc_pages(&mut mapper, addr as u64, size).expect("proc mem grow fail 1545");
    unsafe { allocator.lock().grow(addr, size); };
}

pub fn file_handles() -> Arc<Mutex<BTreeMap<usize, OpenFileHandle>>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.file_handles.clone()
}

/// 进程退出
pub fn exit() {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    syskrnl::allocator::free_pages(proc.code_addr, MAX_PROC_SIZE);
    MAX_PID.fetch_sub(1, Ordering::SeqCst);
    set_id(proc.parent); // FIXME: 因为目前还不存在调度，所以直接设置为父进程
}

pub unsafe fn page_table() -> &'static mut PageTable {
    syskrnl::memory::create_page_table(page_table_frame())
}

/***************************
 *  用户空间相关。祝我们好运！ *
 ***************************/

static CODE_ADDR: AtomicU64 = AtomicU64::new(0);

/// 初始化进程代码地址，在内核初始化的时候调用
pub fn init_process_addr(addr: u64) {
    CODE_ADDR.store(addr, Ordering::SeqCst);
}

impl Process {
    /// 创建进程
    pub fn spawn(bin: &[u8], args_ptr: usize, args_len: usize, args_cap: usize) -> Result<(), ExitCode> {
        if let Ok(id) = Self::create(bin) {
            let mut proc = {
                let table = PROCESS_TABLE.read();
                table[id].clone()
            };
            proc.exec(args_ptr, args_len, args_cap);
            Ok(())
        } else {
            Err(ExitCode::ExecError)
        }
    }

    fn create(bin: &[u8]) -> Result<usize, ()> {
        let page_table_frame = syskrnl::memory::heaped_frame_allocator().allocate_frame().expect("frame alloc failed");
        let page_table = unsafe { syskrnl::memory::create_page_table(page_table_frame) };
        let kernel_page_table = unsafe { syskrnl::memory::active_page_table() };

        for (user_page, kernel_page) in page_table.iter_mut().zip(kernel_page_table.iter()) {
            *user_page = kernel_page.clone();
        }

        let phys_mem_offset = unsafe { syskrnl::memory::PHYS_MEM_OFFSET };
        let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };
        let _kernel_mapper = unsafe { OffsetPageTable::new(kernel_page_table, VirtAddr::new(phys_mem_offset)) };

        // 特别地，打开用户页表的内核使用权限
        unsafe { fix_page_fault_in_userspace(&mut mapper) };

        let proc_size = MAX_PROC_SIZE as u64;
        let kernel_code_addr = CODE_ADDR.fetch_add(proc_size, Ordering::SeqCst);
        let code_addr = kernel_code_addr;
        let stack_addr = code_addr + proc_size - 4096;
        // 紧跟在程序段后面
        debugln!("code_addr:  {:#x}", kernel_code_addr);
        debugln!("stack_addr: {:#x}", stack_addr);

        let mut entry_point = 0;
        let code_ptr = kernel_code_addr as *mut u8;
        let _code_size = bin.len();
        if bin[0..4] == ELF_MAGIC { // 进程代码是ELF格式的
            if let Ok(obj) = object::File::parse(bin) {

                // 先在用户页表上分配
                alloc_pages(&mut mapper, code_addr, proc_size as usize).expect("proc mem alloc 754");
                // // 接下来，把用户页表的地址映射到内核页表上，并在内核页表上分配
                // let user_code_phys_frame = mapper.translate_addr(VirtAddr::new(code_addr)).expect("Map fail 12341");
                // alloc_pages_to_known_phys(&mut kernel_mapper, kernel_code_addr, proc_size as usize, user_code_phys_frame.as_u64(), true).expect("proc mem alloc 564");

                entry_point = obj.entry();
                debugln!("entry_point:{:#x}",entry_point);
                for segment in obj.segments() {
                    let addr = segment.address() as usize;
                    if let Ok(data) = segment.data() {
                        debugln!("before flight? codeaddr,addr,datalen is {:#x},{:#x},{}", code_ptr as usize + addr, addr, data.len());
                        for (i, b) in data.iter().enumerate() {
                            unsafe {
                                //debugln!("code:       {:#x}", code_ptr.add(addr + i) as usize);
                                //debugln!("WRITE: from {:p} to {:p}", b, code_ptr.add(addr + i));
                                core::ptr::write(code_ptr.add(addr + i), *b)
                            }
                        }
                    }
                }
            }
        } else if bin[0..4] == BIN_MAGIC {
            for (i, b) in bin.iter().skip(4).enumerate() {
                unsafe {
                    core::ptr::write(code_ptr.add(i), *b);
                }
            }
        } else { // 文件头错误
            return Err(());
        }

        // 父进程
        let parent = {
            let table = PROCESS_TABLE.read();
            table[id()].clone()
        };

        let data = parent.data.clone();
        let registers = parent.registers;
        let stack_frame = parent.stack_frame;
        let parent = parent.id;

        // 初始化进程的堆分配器
        let mut allocator = LinkedListAllocator::new();
        let heap_addr = PROC_HEAP_ADDR.fetch_add(DEFAULT_HEAP_SIZE, Ordering::SeqCst);

        // 先在用户页表上分配
        alloc_pages(&mut mapper, heap_addr as u64, DEFAULT_HEAP_SIZE).expect("proc heap mem alloc failed 8520");
        // // 再映射到内核页表上
        // let heap_frame = mapper.translate_addr(VirtAddr::new(heap_addr as u64)).expect("map fail 7897");
        // alloc_pages_to_known_phys(&mut kernel_mapper, heap_addr as u64, DEFAULT_HEAP_SIZE, heap_frame.as_u64(), true).expect("proc heap mem alloc failed 3652");

        unsafe { allocator.init(heap_addr, DEFAULT_HEAP_SIZE) };
        let allocator = Arc::new(Locked::new(allocator));

        let id = MAX_PID.fetch_add(1, Ordering::SeqCst);
        let proc = Process {
            id,
            code_addr,
            stack_addr,
            data,
            registers,
            stack_frame,
            entry_point,
            parent,
            allocator,
            page_table_frame,
        };

        let mut table = PROCESS_TABLE.write();
        table[id] = Box::new(proc);

        Ok(id)
    }

    // 切换到用户空间并执行程序
    fn exec(&mut self, args_ptr: usize, args_len: usize, args_cap: usize) {
        //syskrnl::allocator::alloc_pages(heap_addr, 1).expect("proc heap alloc");
        let page_table = unsafe { page_table() };
        let phys_mem_offset = unsafe { syskrnl::memory::PHYS_MEM_OFFSET };
        let _mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };

        // 处理参数
        // 重建指针-长度对
        let ptr_len_pair: Vec<(usize, usize)> = unsafe {
            Vec::from_raw_parts(args_ptr as *mut (usize, usize), args_len, args_cap)
        };
        // 重建参数数组
        let args: Vec<&str> = ptr_len_pair.iter().map(|pair| unsafe {
            let slice = core::slice::from_raw_parts(pair.0 as *const u8, pair.1);
            //debugln!("{:?}",slice);
            core::str::from_utf8(slice).expect("utf8 fail 8547")
        }).collect();
        // 在子进程分配用于存放参数的堆内存
        let mut addr = unsafe {
            self.allocator.lock().alloc(core::alloc::Layout::from_size_align(1024, 1).expect("Layout problem 8741"))
        } as u64;
        // 将参数复制到这些内存上
        let vec: Vec<&str> = args.iter().map(|arg| {
            let ptr = addr as *mut u8;
            addr += arg.len() as u64;
            unsafe {
                let s = core::slice::from_raw_parts_mut(ptr, arg.len());
                s.copy_from_slice(arg.as_bytes());
                //debugln!("{:?}",s);
                core::str::from_utf8_unchecked(s)
            }
        }).collect();
        let align = core::mem::align_of::<&str>() as u64;
        addr += align - (addr % align);
        let args = vec.as_slice();
        let ptr = addr as *mut &str;
        let args: &[&str] = unsafe {
            let s = core::slice::from_raw_parts_mut(ptr, args.len());
            s.copy_from_slice(args);
            s
        };
        let args_ptr = args.as_ptr() as u64;

        SCHEDULER.lock().add(self.clone(), 0);
        // if self.id != 1 {  // 不需要进入环三
        //     return;
        // }
        syskrnl::interrupts::SCHEDULE.store(true, Ordering::SeqCst);

        debugln!("LAUNCH");
        set_id(self.id); // 要换咯！
        // 发射！
        unsafe {
            let (_, flags) = Cr3::read();
            Cr3::write(self.page_table_frame, flags);

            asm!(
            "cli",        // 关中断
            "push {:r}",  // Stack segment (SS)
            "push {:r}",  // Stack pointer (RSP)
            "push 0x200", // RFLAGS with interrupts enabled
            "push {:r}",  // Code segment (CS)
            "push {:r}",  // Instruction pointer (RIP)
            "iretq",
            in(reg) syskrnl::gdt::GDT.1.user_data_selector.0,
            in(reg) self.stack_addr,
            in(reg) syskrnl::gdt::GDT.1.user_code_selector.0,
            in(reg) self.code_addr + self.entry_point,
            in("rdi") args_ptr,
            in("rsi") args.len(),
            );
        }
    }
}
