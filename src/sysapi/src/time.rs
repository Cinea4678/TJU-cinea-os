use serde::{Deserialize, Serialize};
use ufmt::uDebug;

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