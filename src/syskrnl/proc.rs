use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use lazy_static::lazy_static;
use object::{Object, ObjectSegment};
use spin::RwLock;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::VirtAddr;
use crate::sysapi::proc::ExitCode;
use crate::syskrnl;

const MAX_FILE_HANDLES: usize = 64;
/// 最大进程数，先写2个，后面再改
const MAX_PROCS: usize = 2;
const MAX_PROC_SIZE: usize = 10 << 20;

pub static PID: AtomicUsize = AtomicUsize::new(0);
pub static MAX_PID: AtomicUsize = AtomicUsize::new(1);

lazy_static! {
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
    //file_handles: [Option<Box<Resource>>; MAX_FILE_HANDLES],
}

#[repr(align(8), C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

#[derive(Clone, Debug)]
pub struct Process {
    id: usize,
    code_addr: u64,
    stack_addr: u64,
    entry_point: u64,
    stack_frame: InterruptStackFrameValue,
    registers: Registers,
    data: ProcessData,
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        // let mut file_handles = [(); MAX_FILE_HANDLES].map(|_| None);
        // file_handles[0] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdin
        // file_handles[1] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdout
        // file_handles[2] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stderr
        // file_handles[3] = Some(Box::new(Resource::Device(Device::Null))); // stdnull
        Self { env, dir, user /*, file_handles*/ }
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
            registers: Registers::default(),
            data: ProcessData::new("/", None),
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

/// 进程退出
pub fn exit() {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    syskrnl::allocator::free_pages(proc.code_addr, MAX_PROC_SIZE);
    MAX_PID.fetch_sub(1, Ordering::SeqCst);
    set_id(0); // FIXME: 因为目前还不存在调度，所以直接设置为0
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
    pub fn spawn(bin: &[u8], args_ptr: usize, args_len: usize) -> Result<(), ExitCode> {
        if let Ok(id) = Self::create(bin) {
            let proc = {
                let table = PROCESS_TABLE.read();
                table[id].clone()
            };
            proc.exec(args_ptr, args_len);
            Ok(())
        } else {
            Err(ExitCode::ExecError)
        }
    }

    fn create(bin: &[u8]) -> Result<usize, ()> {
        let proc_size = MAX_PROC_SIZE as u64;
        let code_addr = CODE_ADDR.fetch_add(proc_size, Ordering::SeqCst);
        let stack_addr = code_addr + proc_size; // 紧跟在程序段后面

        let mut entry_point = 0;
        let code_ptr = code_addr as *mut u8;
        if bin[0..4] == ELF_MAGIC { // 进程代码是ELF格式的
            if let Ok(obj) = object::File::parse(bin) {
                entry_point = obj.entry();
                for segment in obj.segments() {
                    let addr = segment.address() as usize;
                    if let Ok(data) = segment.data() {
                        for (i, b) in data.iter().enumerate() {
                            unsafe {
                                core::ptr::write(code_ptr.add(addr + i), *b)
                            }
                        }
                    }
                }
            }
        } else if bin[0..4] == BIN_MAGIC {
            for (i, b) in bin.iter().enumerate() {
                unsafe {
                    core::ptr::write(code_ptr.add(i), *b);
                }
            }
        } else { // 文件头错误
            return Err(());
        }

        let parent = {
            let table = PROCESS_TABLE.read();
            table[id()].clone()
        }; // 父进程

        let data = parent.data.clone();
        let registers = parent.registers;
        let stack_frame = parent.stack_frame;

        let id = MAX_PID.fetch_add(1, Ordering::SeqCst);
        let proc = Process {
            id,
            code_addr,
            stack_addr,
            data,
            registers,
            stack_frame,
            entry_point,
        };

        let mut table = PROCESS_TABLE.write();
        table[id] = Box::new(proc);

        Ok(id)
    }

    // 切换到用户空间并执行程序
    fn exec(&self, args_ptr: usize, args_len: usize) {
        let heap_addr = self.code_addr + (self.stack_addr - self.code_addr) / 2;
        syskrnl::allocator::alloc_pages(heap_addr, 1).expect("proc heap alloc");

        let args_ptr = ptr_from_addr(args_ptr as u64) as usize;
        let args: &[&str] = unsafe {
            core::slice::from_raw_parts(args_ptr as *const &str, args_len)
        };
        let mut addr = heap_addr;
        let vec: Vec<&str> = args.iter().map(|arg| {
            let ptr = addr as *mut u8;
            addr += arg.len() as u64;
            unsafe {
                let s = core::slice::from_raw_parts_mut(ptr, arg.len());
                s.copy_from_slice(arg.as_bytes());
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

        set_id(self.id); // 要换咯！
        // 发射！
        unsafe {
            asm!(
            "cli",      // 关中断
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
            in("rsi") args_len,
            );
        }
    }
}
