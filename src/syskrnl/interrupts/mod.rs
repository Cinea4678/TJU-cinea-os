use alloc::format;
use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, InterruptStackFrameValue, PageFaultErrorCode};

use crate::syskrnl::gui::panic;
use crate::syskrnl::io::qemu::qemu_print;
use crate::syskrnl::proc::{Registers, SCHEDULER};
use crate::syskrnl::time;
use crate::syskrnl::time::ticks;
use crate::{debugln, println, syskrnl};

pub mod pics;

/// 将IRQ转换为中断号码
fn interrupt_index(irq: u8) -> u8 {
    pics::PIC_1_OFFSET + irq
}

/// 默认IRQ处理器
fn default_irq_handler() {}

lazy_static! {
    pub static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = Mutex::new([default_irq_handler; 16]);

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint
            .set_handler_fn(breakpoint_handler);
        unsafe{
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(syskrnl::gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(syskrnl::gdt::PAGE_FAULT_IST_INDEX);
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault_handler)
                .set_stack_index(syskrnl::gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);
            // PIT刷新中断
            idt[interrupt_index(0) as usize]
                .set_handler_fn(core::mem::transmute(wrapped_clock_handler as *mut fn()));
            // 系统调用接口
            idt[0x80]
                .set_handler_fn(core::mem::transmute(wrapped_syscall_handler as *mut fn()))
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
            // 特殊调用接口：保存进程状态
            idt[0x81]
                .set_handler_fn(core::mem::transmute(wrapped_save_context as *mut fn()))
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
            // 特殊调用接口：进程等待事件
            idt[0x82]
                .set_handler_fn(core::mem::transmute(wrapped_proc_wait as *mut fn()))
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }
        idt[interrupt_index(1) as usize].set_handler_fn(irq1_handler);
        idt[interrupt_index(2) as usize].set_handler_fn(irq2_handler);
        idt[interrupt_index(3) as usize].set_handler_fn(irq3_handler);
        idt[interrupt_index(4) as usize].set_handler_fn(irq4_handler);
        idt[interrupt_index(5) as usize].set_handler_fn(irq5_handler);
        idt[interrupt_index(6) as usize].set_handler_fn(irq6_handler);
        idt[interrupt_index(7) as usize].set_handler_fn(irq7_handler);
        idt[interrupt_index(8) as usize].set_handler_fn(irq8_handler);
        idt[interrupt_index(9) as usize].set_handler_fn(irq9_handler);
        idt[interrupt_index(10) as usize].set_handler_fn(irq10_handler);
        idt[interrupt_index(11) as usize].set_handler_fn(irq11_handler);
        idt[interrupt_index(12) as usize].set_handler_fn(irq12_handler);
        idt[interrupt_index(13) as usize].set_handler_fn(irq13_handler);
        idt[interrupt_index(14) as usize].set_handler_fn(irq14_handler);
        idt[interrupt_index(15) as usize].set_handler_fn(irq15_handler);
        idt
    };
}

/// 参考moros
macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            unsafe {
                pics::PICS.lock().notify_end_of_interrupt(interrupt_index($irq));
            }
        }
    };
}

irq_handler!(irq0_handler, 0);
irq_handler!(irq1_handler, 1);
irq_handler!(irq2_handler, 2);
irq_handler!(irq3_handler, 3);
irq_handler!(irq4_handler, 4);
irq_handler!(irq5_handler, 5);
irq_handler!(irq6_handler, 6);
irq_handler!(irq7_handler, 7);
irq_handler!(irq8_handler, 8);
irq_handler!(irq9_handler, 9);
irq_handler!(irq10_handler, 10);
irq_handler!(irq11_handler, 11);
irq_handler!(irq12_handler, 12);
irq_handler!(irq13_handler, 13);
irq_handler!(irq14_handler, 14);
irq_handler!(irq15_handler, 15);

pub fn set_irq_handler(irq: u8, handler: fn()) {
    let mut handlers = IRQ_HANDLERS.lock();
    handlers[irq as usize] = handler;
}

pub fn init_idt() {
    IDT.load();
}

/// 调试异常处理函数
extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", _stack_frame);
    qemu_print(format!("EXCEPTION: BREAKPOINT\n{:#?}\n", _stack_frame).as_str());
}

/// 双重异常处理函数
extern "x86-interrupt" fn double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", _stack_frame);
    qemu_print(format!("EXCEPTION: DOUBLE FAULT\n{:#?}\n", _stack_frame).as_str());
    loop {}
}

/// 一般保护异常处理函数
extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    debugln!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nStack Frame: {:#?}\nError: {:?}\n",
        stack_frame,
        error_code
    );

    let panic_desc = format!("Stack Frame: {:#?}\nError: {:?}\n", stack_frame, error_code);
    let panic_info = panic::PanicInfo::new("一般保护异常 General Protection", panic_desc.as_str());

    panic::handle_panic(&panic_info);
}

/// 页错异常处理函数
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    qemu_print(format!("EXCEPTION: PAGE FAULT\n").as_str());
    qemu_print(format!("Accessed Address: {:?}\n", Cr2::read()).as_str());
    qemu_print(format!("{:#?}\n", stack_frame).as_str());

    let panic_desc = format!("Accessed Address: {:?}\n{:#?}\n", Cr2::read(), stack_frame);
    let panic_info = panic::PanicInfo::new("页错异常 Page Fault", panic_desc.as_str());

    panic::handle_panic(&panic_info);
}

// 裸函数包装器，用于把暂存寄存器的值保存到堆栈
// See（不是第一版怎么还有这种好东西）: https://os.phil-opp.com/returning-from-exceptions/#a-naked-wrapper-function
macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        pub unsafe extern "sysv64" fn $w() {
            asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rbx",
                "push rbp",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                "mov rsi, rsp", // Arg #2: register list
                "mov rdi, rsp", // Arg #1: interupt frame
                "add rdi, 15 * 8",
                "call {}",
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rbp",
                "pop rbx",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "iretq",
                sym $fn,
                options(noreturn)
            );
        }
    };
}

wrap!(syscall_handler => wrapped_syscall_handler);

extern "sysv64" fn syscall_handler(stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    // The registers order follow the System V ABI convention
    let n = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    if n == cinea_os_sysapi::call::SPAWN {
        // 保存现场
        syskrnl::proc::set_stack_frame(**stack_frame);
        syskrnl::proc::set_registers(*regs);
    }

    let res = syskrnl::syscall::dispatcher(n, arg1, arg2, arg3, arg4);

    if n == cinea_os_sysapi::call::EXIT {
        // 恢复现场
        debugln!("恢复现场");
        debugln!("额外信息：{:?}", SCHEDULER.lock());
        let next_pid = res;
        unsafe {
            switch_context_to(next_pid, stack_frame, regs);
        }
    } else {
        regs.rax = res;
    }

    unsafe { pics::PICS.lock().notify_end_of_interrupt(0x80) };
}

unsafe fn switch_context_to(pid: usize, stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    syskrnl::proc::set_id(pid);
    let sf = syskrnl::proc::stack_frame();
    //stack_frame.as_mut().write(sf);
    let (_, flags) = Cr3::read();
    Cr3::write(syskrnl::proc::page_table_frame(), flags);
    core::ptr::write_volatile(stack_frame.as_mut().extract_inner() as *mut InterruptStackFrameValue, sf); // FIXME
    core::ptr::write_volatile(regs, syskrnl::proc::registers());
}

pub static SCHEDULE: AtomicBool = AtomicBool::new(false);
static LAST_SCHEDULE: AtomicUsize = AtomicUsize::new(0);
pub static NO_SCHEDULE: AtomicBool = AtomicBool::new(false);

wrap!(clock_handler => wrapped_clock_handler);

/// 时钟中断处理程序
extern "sysv64" fn clock_handler(stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    // 先把时钟发过去
    let handlers = IRQ_HANDLERS.lock();
    handlers[0]();

    if let Some(pid) = time::check_wakeup() {
        SCHEDULER.lock().wakeup(pid);
    }

    if SCHEDULE.load(Ordering::SeqCst) && ticks() - LAST_SCHEDULE.load(Ordering::SeqCst) > 10 {
        let mut schedule = || {
            if NO_SCHEDULE.load(Ordering::SeqCst) {
                if ticks() - LAST_SCHEDULE.load(Ordering::SeqCst) > 1000 {
                    // 强行恢复调度
                    NO_SCHEDULE.store(false, Ordering::SeqCst);
                } else {
                    return;
                }
            }

            let next_pid = SCHEDULER.lock().timeup();
            // debugln!("Schedule: next_pid = {}; now_pid = {}", next_pid, syskrnl::proc::id());

            if next_pid != syskrnl::proc::id() {
                syskrnl::proc::set_stack_frame(**stack_frame);
                syskrnl::proc::set_registers(*regs);

                unsafe {
                    switch_context_to(next_pid, stack_frame, regs);
                }
            }

            let lock = syskrnl::event::EVENT_DATA.lock();
            if lock.contains_key(&next_pid) {
                regs.rax = *lock.get(&next_pid).unwrap();
            }
            drop(lock);

            LAST_SCHEDULE.store(ticks(), Ordering::SeqCst);
        };

        schedule();
    }

    unsafe { pics::PICS.lock().notify_end_of_interrupt(interrupt_index(0) as u8) };
}

wrap!(save_context => wrapped_save_context);

extern "sysv64" fn save_context(stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    syskrnl::proc::set_stack_frame(**stack_frame);
    syskrnl::proc::set_registers(*regs);

    unsafe { pics::PICS.lock().notify_end_of_interrupt(interrupt_index(0x81) as u8) };
}

wrap!(proc_wait => wrapped_proc_wait);

extern "sysv64" fn proc_wait(stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    // The registers order follow the System V ABI convention
    let n = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    // 保存现场
    syskrnl::proc::set_stack_frame(**stack_frame);
    syskrnl::proc::set_registers(*regs);

    let next_pid = syskrnl::event::dispatcher(n, arg1, arg2, arg3, arg4);

    // 恢复现场
    unsafe {
        switch_context_to(next_pid, stack_frame, regs);
    }

    unsafe { pics::PICS.lock().notify_end_of_interrupt(0x82) };
}
