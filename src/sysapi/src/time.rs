use serde::{Deserialize, Serialize};
use ufmt::uDebug;
use crate::call::READ_TIME;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct Time {
    pub hour: u16,
    pub min: u16,
    pub sec: u16,
    pub millis: u16,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl Date {
    pub fn from_fatfs(date: &fatfs::Date) -> Self {
        Self {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }

    pub fn new(year: u16, month: u16, day: u16) -> Self {
        Self { year, month, day }
    }
}

impl uDebug for Date {
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
        where
            W: ufmt::uWrite + ?Sized {
        w.debug_struct("Date")?
            .field("year", &self.year)?
            .field("month", &self.month)?
            .field("day", &self.day)?
            .finish()?;
        Ok(())
    }
}

impl Time {
    pub fn from_fatfs(time: &fatfs::Time) -> Self {
        Self {
            millis: time.millis,
            sec: time.sec,
            min: time.min,
            hour: time.hour,
        }
    }

    pub fn new(hour: u16, min: u16, sec: u16, millis: u16) -> Self {
        Self { hour, min, sec, millis }
    }
}

impl uDebug for Time {
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
        where
            W: ufmt::uWrite + ?Sized {
        w.debug_struct("Time")?
            .field("hour", &self.hour)?
            .field("min", &self.min)?
            .field("sec", &self.sec)?
            .field("millis", &self.millis)?
            .finish()?;
        Ok(())
    }
}

impl DateTime {
    pub fn from_fatfs(datetime: &fatfs::DateTime) -> Self {
        Self {
            date: Date::from_fatfs(&datetime.date),
            time: Time::from_fatfs(&datetime.time),
        }
    }

    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }
}

impl uDebug for DateTime {
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
        where
            W: ufmt::uWrite + ?Sized {
        w.debug_struct("DateTime")?
            .field("date", &self.date)?
            .field("time", &self.time)?
            .finish()?;
        Ok(())
    }
}

pub fn get_datetime() -> DateTime {
    let ret: Result<DateTime, _> = syscall_with_deserialize!(READ_TIME);
    ret.expect("Read time failed. 8d76")
}
