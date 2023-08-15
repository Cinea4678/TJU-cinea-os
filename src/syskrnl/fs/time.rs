use core::fmt::{Debug, Formatter};

use fatfs::{Date, DateTime, Time};

use crate::syskrnl;

pub struct CosTimeProvider;

impl Debug for CosTimeProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Cinea-OS Time Provider")
    }
}

impl fatfs::TimeProvider for CosTimeProvider {
    fn get_current_date(&self) -> Date {
        let now = syskrnl::time::raw_time();
        Date::new(now.year as u16, now.month as u16, now.day as u16)
    }

    fn get_current_date_time(&self) -> DateTime {
        let now = syskrnl::time::raw_time();
        let date = Date::new(now.year as u16, now.month as u16, now.day as u16);
        let time = Time::new(now.hour as u16, now.minute as u16, now.second as u16, 0);
        DateTime::new(date, time)
    }
}
