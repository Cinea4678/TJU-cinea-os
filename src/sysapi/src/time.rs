use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct Time {
    pub millis: u16,
    pub sec: u16,
    pub min: u16,
    pub hour: u16,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl Date{
    pub fn from_fatfs(date: &fatfs::Date)->Self {
        Self{
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }
}

impl Time{
    pub fn from_fatfs(time: &fatfs::Time)->Self {
        Self{
            millis: time.millis,
            sec: time.sec,
            min: time.min,
            hour: time.hour,
        }
    }
}

impl DateTime{
    pub fn from_fatfs(datetime: &fatfs::DateTime)->Self {
        Self{
            date: Date::from_fatfs(&datetime.date),
            time: Time::from_fatfs(&datetime.time),
        }
    }
}