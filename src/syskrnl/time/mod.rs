use cinea_os_sysapi::fs::FileIO;
use x86_64::instructions::interrupts;

pub use datetime::*;
pub use pit::PIT_PER_SECOND as TICKS_PER_SECOND;
pub use sleep::{add_sleep, check_wakeup};

use crate::syskrnl::time::cmos::{RawTime, read_rtc};

pub mod cmos;
mod pit;
pub mod tsc;
mod datetime;
mod sleep;

const TIME_ZONE: u8 = 8;

/// 获取RTC时间
pub fn raw_time() -> RawTime {
    let mut tm = read_rtc();
    tm.hour += TIME_ZONE;
    if tm.hour >= 24 {
        tm.hour -= 24;
        tm.day += 1;
    }
    tm
}

/// 获取两个tick之间的时间间隔
pub fn time_between_ticks() -> f64 {
    pit::time_between_ticks()
}

pub fn last_rtc_update() -> usize {
    unimplemented!()
}

/// 获取启动后经过的Tick数
pub fn ticks() -> usize {
    pit::get_ticks()
}

/// 获取系统启动到现在的时间
pub fn uptime() -> f64 {
    pit::get_uptime()
}

/// Halt
pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

/// 等待指定的秒数（低精度）
pub fn sleep(seconds: f64) {
    let wakeup_time = uptime() + seconds;
    while uptime() < wakeup_time {
        halt();
    }
}

/// 等待指定的纳秒数（高精度）
pub fn nanowait(nanoseconds: u64) {
    tsc::nanowait(nanoseconds);
}

pub fn init() {
    pit::init();
    cmos::init();
    tsc::init();
    sleep::init();
}

pub struct UpTimeDevice;

impl FileIO for UpTimeDevice {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let uptime = uptime();
        let slice = uptime.to_le_bytes();
        buf.copy_from_slice(&slice);
        Ok(8)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Ok(0)
    }
}
