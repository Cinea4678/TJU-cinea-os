// in src/vga_buffer/mod.rs

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;
use crate::println;

/// VGA标准颜色
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// 两位颜色码，在内部使用
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// 提交到内存中的VGA字符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const TAB_SIZE: usize = 4;

/// VGA屏幕
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// 输出器
pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            0x08 => self.backspace(), // \b
            b'\t' => self.horizontal_tab(),
            b'\n' => self.new_line(),
            b'\r' => self.carriage_return(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position.clone();
                let col = self.column_position.clone();
                let color_code = self.color_code.clone();

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\r' | b'\t' | 0x08 => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row.clone()][col].write(blank);
        }
    }

    fn new_line(&mut self) {
        self.row_position += 1;
        self.column_position = 0;

        if self.row_position >= BUFFER_HEIGHT {
            // 向上滚屏
            for row in 0..BUFFER_HEIGHT - 1 {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[row.clone()][col.clone()].write(self.buffer.chars[row.clone() + 1][col.clone()].read());
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.row_position = BUFFER_HEIGHT - 1;
        }
    }

    fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
        }
    }

    fn carriage_return(&mut self) {
        self.column_position = 0;
    }

    fn horizontal_tab(&mut self) {
        self.column_position += TAB_SIZE - (self.column_position.clone() % TAB_SIZE);
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::LightCyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    // 防止死锁
    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).unwrap();
    })
}

// #[macro_export]
// macro_rules! vga_print {
//     ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
// }
// 
// #[macro_export]
// macro_rules! vga_println {
//     () => ($crate::print!("\n"));
//     ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
// }

pub fn print_something() {
    println!("Every smallest dream matters.\n\n");
    println!("\t----Hello World From Cinea's Operating System\n");
    println!("\t\t\t\t\t\t\t\t2023.5.30\n");
}