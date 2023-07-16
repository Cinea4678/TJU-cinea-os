use core::sync::atomic::{AtomicUsize, Ordering};
// 从CMOS读取日期和时间
use x86::io::{inb, outb};
use x86_64::instructions::interrupts;
use crate::{syskrnl};
use crate::syskrnl::interrupts::set_irq_handler;

const CURRENT_YEAR: u32 = 2023;

static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);

#[repr(u16)]
enum CmosPort {
    Address = 0x70u16,
    Data = 0x71,
}

#[repr(u8)]
enum Interrupt {
    /// 周期模式
    Periodic = 1 << 6,
    /// 闹钟模式
    Alarm = 1 << 5,
    /// 更新模式
    Update = 1 << 4,
}

fn get_update_in_progress_flag() -> u8 {
    unsafe {
        outb(CmosPort::Address as u16, 0x0A);
        inb(CmosPort::Data as u16) & 0x80
    }
}

fn get_rtc_register(reg: u8) -> u8 {
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
    time.second = get_rtc_register(0x00);
    time.minute = get_rtc_register(0x02);
    time.hour = get_rtc_register(0x04);
    time.day = get_rtc_register(0x07);
    time.month = get_rtc_register(0x08);
    time.year = get_rtc_register(0x09) as u32;

    // 第二次获取时间
    loop {
        let last_time = time;
        while get_update_in_progress_flag() > 0 {}
        time.second = get_rtc_register(0x00);
        time.minute = get_rtc_register(0x02);
        time.hour = get_rtc_register(0x04);
        time.day = get_rtc_register(0x07);
        time.month = get_rtc_register(0x08);
        time.year = get_rtc_register(0x09) as u32;
        if time == last_time { break; }
    }

    // 处理时间格式
    let register_b = get_rtc_register(0x0B);
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

/// 禁用NMI（非屏蔽中断）
fn disable_nmi() {
    unsafe {
        let prev = inb(CmosPort::Address as u16);
        outb(CmosPort::Address as u16, prev | 0x80);
    }
}

/// 启用NMI（非屏蔽中断）
fn enable_nmi() {
    unsafe {
        let prev = inb(CmosPort::Address as u16);
        outb(CmosPort::Address as u16, prev & 0x7F);
    }
}

/// 通知CMOS中断已经处理结束
pub fn notify_end_of_interrupt() {
    unsafe {
        outb(CmosPort::Address as u16, 0x0C);
        inb(CmosPort::Data as u16);
    }
}

/// 启用CMOS中断
fn enable_interrupt(interrupt: Interrupt) {
    interrupts::without_interrupts(|| {
        disable_nmi();
        unsafe {
            outb(CmosPort::Address as u16, 0x0B);
            let prev = inb(CmosPort::Data as u16);
            outb(CmosPort::Address as u16, 0x0B);
            outb(CmosPort::Data as u16, prev | interrupt as u8);
        }
        enable_nmi();
        notify_end_of_interrupt();
    });
}

/// 启用周期中断
pub fn enable_periodic_interrupt() {
    enable_interrupt(Interrupt::Periodic);
}

/// 启用闹钟中断
pub fn enable_alarm_interrupt() {
    enable_interrupt(Interrupt::Alarm);
}

/// 启用更新中断
pub fn enable_update_interrupt() {
    enable_interrupt(Interrupt::Update);
}

/// 设置周期中断的频率
///
/// `rate`必须介于3到15之间
///
/// 效果是： `32768 >> (rate - 1)`
pub fn set_periodic_interrupt_rate(rate: u8) {
    interrupts::without_interrupts(|| {
        disable_nmi();
        unsafe {
            outb(CmosPort::Address as u16, 0x0A);
            let prev = inb(CmosPort::Data as u16);
            outb(CmosPort::Address as u16, 0x0A);
            outb(CmosPort::Data as u16, (prev & 0xF0) | rate);
        }
        enable_nmi();
        notify_end_of_interrupt();
    });
}

/// RTC中断处理函数
pub fn rtc_interrupt_handler() {
    let status = unsafe {
        outb(CmosPort::Address as u16, 0x0C);
        inb(CmosPort::Data as u16)
    };
    if status & Interrupt::Periodic as u8 > 0 {
        // 周期中断
        syskrnl::clock::half_sec_handler();
    } else if status & Interrupt::Update as u8 > 0 {
        // 更新中断
        LAST_RTC_UPDATE.store(super::ticks(), Ordering::Relaxed);
    }
    notify_end_of_interrupt();
}

pub fn init() {
    set_irq_handler(8, rtc_interrupt_handler);
    set_periodic_interrupt_rate(15); // 拉满
    enable_periodic_interrupt();
    enable_update_interrupt();
}