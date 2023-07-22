use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod pci;
pub mod qemu;
pub mod mouse;
pub mod ata;

pub enum VideoMode {
    Text,
    Graphic,
}

impl VideoMode {
    pub fn is_text(&self) -> bool {
        match self {
            VideoMode::Text => true,
            VideoMode::Graphic => false,
        }
    }

    pub fn set_graphic(&mut self) {
        *self = VideoMode::Graphic;
    }
}

lazy_static! {
    pub static ref VIDEO_MODE : Mutex<VideoMode> = Mutex::new(VideoMode::Text);
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::syskrnl::io::qemu::_qemu_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debugln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::debug!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // 向Qemu也发送一份
    qemu::_qemu_print(args);

    if VIDEO_MODE.lock().is_text() {
        crate::syskrnl::vga_buffer::_print(args);
    } else {
        crate::syskrnl::graphic::_print(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::syskrnl::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
