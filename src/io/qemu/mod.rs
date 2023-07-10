use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use x86::io::outb;
use x86_64::instructions::interrupts;
use spin::Mutex;

#[repr(u16)]
enum IoPort {
    Com1 = 0x3F8
}

pub fn qemu_print(content: &str) {
    for ch in content.as_bytes() {
        unsafe { outb(IoPort::Com1 as u16, *ch); };
    }
}

struct QemuWriter();

impl fmt::Write for QemuWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        qemu_print(s);
        Ok(())
    }
}

lazy_static! {
    static ref QEMU_WRITER: Mutex<QemuWriter> = Mutex::new(QemuWriter());
}

#[doc(hidden)]
pub fn _qemu_print(args: fmt::Arguments) {
    // 防止死锁
    interrupts::without_interrupts(|| {
        QEMU_WRITER.lock().write_fmt(args).unwrap();
    })
}