use lazy_static::lazy_static;
use pc_keyboard::{Keyboard, layouts};
use spin::lock_api::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use pics::InterruptIndex;

use crate::{print, println};

pub mod pics;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(time_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

/// 调试异常处理函数
extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", _stack_frame);
}

/// 双重异常处理函数
extern "x86-interrupt" fn double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", _stack_frame);
    loop {}
}

/// 定时器中断处理函数
extern "x86-interrupt" fn time_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        pics::PICS.lock().notify_end_of_interrupt(pics::InterruptIndex::Timer.as_u8());
    }
}

/// 键盘中断处理函数
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}",character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        pics::PICS.lock().notify_end_of_interrupt(pics::InterruptIndex::Keyboard.as_u8());
    }
}

/// 页错异常处理函数
extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    use crate::hlt_loop;
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("{:#?}", _stack_frame);
    hlt_loop();
}
