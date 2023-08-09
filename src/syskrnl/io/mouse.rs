use x86::io::{inb, outb};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;
use crate::syskrnl;
use crate::syskrnl::task::mouse::mouse_interrupt_handler;

const ACK: u8 = 0xFA;

fn send_command(cmd: u8) {
    unsafe {
        outb(0x64, 0xD4);
        outb(0x60, cmd);
    }
    if cmd != 0xFF {
        while unsafe { inb(0x60) } != ACK {}
    } else {
        while unsafe { inb(0x60) } != 0xAA {}
    }
}

pub fn init(){
    // 开启鼠标设备
    let mut port_64 = Port::<u8>::new(0x64);
    let mut port_60 = Port::<u8>::new(0x60);

    // 等待端口0x64状态字节的第二个比特为0、第一个比特为1
    while unsafe { port_64.read() } & 0x2 != 0 {}

    unsafe {
        port_64.write(0xA8); // 开启鼠标
        port_64.write(0x20); // 读取当前设置
        while { port_64.read() } & 0x1 == 0 {}
        let status = port_60.read();
        port_64.write(0x60); // 设置命令字节
        port_60.write(status | 0x2); // 设置第二个比特
    }

    send_command(0xF6);
    send_command(0xF4);
    send_command(0xF3);
    send_command(40);

    interrupts::without_interrupts(|| {
        syskrnl::interrupts::set_irq_handler(12, mouse_interrupt_handler);
    });
}