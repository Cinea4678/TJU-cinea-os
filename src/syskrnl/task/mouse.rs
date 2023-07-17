use x86::io::{inb, outb};
use x86_64::instructions::interrupts;
use crate::syskrnl;

/// 键盘中断处理函数
fn mouse_interrupt_handler() {
    debugln!("Mouse Event!");
}

fn send_command(cmd: u8) {
    unsafe {
        while inb(0x64) & 0x2 != 0 {}
        outb(0x64, cmd);
    }
}

fn send_data(data: u8) {
    unsafe {
        while inb(0x64) & 0x2 != 0 {}
        outb(0x60, data);
    }
}

fn read_data() -> u8 {
    unsafe {
        while inb(0x64) & 0x1 == 0 {}
        inb(0x60)
    }
}

pub fn init(){
    // 启用鼠标
    //send_command(0xA8);
    // 读取命令字节
    send_command(0x20);
    let mut command_byte = read_data();
    // 启用IRQ
    command_byte |= 0x2;
    command_byte &= !0x20;
    // 写回命令字节
    send_command(0x60);
    send_data(command_byte);
    // 启用数据报告
    //send_data(0xF4);

    interrupts::without_interrupts(|| {
        syskrnl::interrupts::set_irq_handler(12, mouse_interrupt_handler);
    });
}