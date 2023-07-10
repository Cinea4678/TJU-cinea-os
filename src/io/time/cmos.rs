// 从CMOS读取日期和时间

use alloc::format;

use x86::io::{inb, outb};

use crate::io::qemu::qemu_print;

const CURRENT_YEAR: u32 = 2023;

#[repr(u16)]
enum CmosPort {
    Address = 0x70u16,
    Data = 0x71,
}

fn get_update_in_progress_flag() -> u8 {
    unsafe {
        outb(CmosPort::Address as u16, 0x0A);
        inb(CmosPort::Data as u16) & 0x80
    }
}

fn get_RTC_register(reg: u8) -> u8 {
    unsafe {
        outb(CmosPort::Address as u16, reg);
        inb(CmosPort::Data as u16)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct RawTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u32,
}

pub fn read_rtc() -> RawTime {
    let mut time = RawTime::default();

    // 第一次获取时间
    while get_update_in_progress_flag() > 0 {}
    time.second = get_RTC_register(0x00);
    time.minute = get_RTC_register(0x02);
    time.hour = get_RTC_register(0x04);
    time.day = get_RTC_register(0x07);
    time.month = get_RTC_register(0x08);
    time.year = get_RTC_register(0x09) as u32;

    // 第二次获取时间
    loop {
        let last_time = time;
        while get_update_in_progress_flag() > 0 {}
        time.second = get_RTC_register(0x00);
        time.minute = get_RTC_register(0x02);
        time.hour = get_RTC_register(0x04);
        time.day = get_RTC_register(0x07);
        time.month = get_RTC_register(0x08);
        time.year = get_RTC_register(0x09) as u32;
        if time == last_time { break; }
    }

    // 处理时间格式
    let register_b = get_RTC_register(0x0B);
    if (register_b & 0x04) == 0 {
        time.second = (time.second & 0x0F) + ((time.second / 16) * 10);
        time.minute = (time.minute & 0x0F) + ((time.minute / 16) * 10);
        time.hour = ((time.hour & 0x0F) + (((time.hour & 0x70) / 16) * 10)) | (time.hour & 0x80);
        time.day = (time.day & 0x0F) + ((time.day / 16) * 10);
        time.month = (time.month & 0x0F) + ((time.month / 16) * 10);
        time.year = (time.year & 0x0F) + ((time.year / 16) * 10);
    }

    // 如有必要，转换为24小时制
    if (register_b & 0x02) == 0 && (time.hour & 0x80) > 0 {
        time.hour = ((time.hour & 0x7F) + 12) % 24;
    }

    // 计算完整的4位数年份
    time.year += CURRENT_YEAR - (CURRENT_YEAR % 100);

    time
}

