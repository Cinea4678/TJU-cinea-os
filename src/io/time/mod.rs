use crate::io::time::cmos::{RawTime, read_RTC};

pub mod cmos;

const TIME_ZONE: u8 = 8;

pub fn get_raw_time() -> RawTime {
    let mut tm = read_RTC();
    tm.hour += TIME_ZONE;
    tm
}