use x86::io::{inb, outb};
use crate::debugln;

pub fn init_mouse(){
    unsafe {
        // 发送Get-Compaq-Status
        outb(0x64,0x20);
        // 接受鼠标状态码
        let mut status = inb(0x64);
        debugln!("Status:{}",status);
        // 设置鼠标状态码
        status |= 0x02;
        status &= !0x20;
        // 发送Set-Compaq-Status
        outb(0x64,0x60);
        outb(0x60,status);
    }
}