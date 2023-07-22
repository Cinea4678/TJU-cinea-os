use crate::syskrnl::time::cmos::RawTime;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct DateTime {
    pub millisecond: u16,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u32,
}

impl DateTime {
    pub fn from_raw_time(rt: &RawTime) -> Self {
        DateTime {
            millisecond: 0,
            second: rt.second,
            minute: rt.minute,
            hour: rt.hour,
            day: rt.day,
            month: rt.month,
            year: rt.year,
        }
    }

    pub fn add_year(&mut self, years: u64) {
        self.year += years as u32;
    }

    pub fn add_month(&mut self, months: u64) {
        self.month += months as u8;
        if self.month > 12 {
            self.month = (months - 1) as u8 % 12 + 1;
            self.add_year((months - 1) / 12);
        }
    }
}