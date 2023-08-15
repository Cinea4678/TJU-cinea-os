use alloc::collections::{BTreeSet};
use core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;
use cinea_os_sysapi::event::{gui_event_make_ret, GUI_EVENT_TIME_UPDATE};
use crate::syskrnl;
use crate::syskrnl::event::EVENT_QUEUE;
use crate::syskrnl::proc::SCHEDULER;
use crate::syskrnl::time;

const DAYS_BEFORE_MONTH: [u64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

const UNIX_EPOCH_OFFSET: u64 = 1030770000; // 2002-08-31 13:00:00 CST ( 2002-08-31 05:00:00 UTC )

/// Cinea戳转Unix戳
pub fn cinea_epoch_to_unix_epoch(cinea_epoch: u64) -> u64 {
    cinea_epoch + UNIX_EPOCH_OFFSET
}

/// Unix戳转Cinea戳
pub fn unix_epoch_to_cinea_epoch(unix_epoch: u64) -> u64 {
    unix_epoch - UNIX_EPOCH_OFFSET
}

fn is_leap_year(year: u32) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
}

fn days_before_year(year: u32) -> u64 {
    (2003..year).fold(0, |days, y| {
        days + if is_leap_year(y) { 366 } else { 365 }
    })
}

fn days_before_month(year: u32, month: u8) -> u64 {
    let leap_day = is_leap_year(year) && month > 2;
    DAYS_BEFORE_MONTH[(month as usize) - 1] + (leap_day as u64)
}

pub fn realtime() -> f64 {
    let raw_time = time::raw_time();

    // 先算到2003-1-1 GMT的秒数
    let timestamp = 86400 * days_before_year(raw_time.year)
        + 86400 * days_before_month(raw_time.year, raw_time.month)
        + 84600 * (raw_time.day - 1) as u64
        + 3600 * raw_time.hour as u64
        + 60 * raw_time.minute as u64
        + raw_time.second as u64
        + 10540800 + 68400;

    let fract = time::time_between_ticks() * (time::ticks() - time::last_rtc_update()) as f64;

    timestamp as f64 + fract
}

lazy_static! {
    pub static ref GUI_TIME_UPDATE_EVENT_NEEDER: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());
}

static EVEN_ODD_FLAG: AtomicBool = AtomicBool::new(false);

/// 半秒事件处理器
pub fn half_sec_handler() {
    let eof = EVEN_ODD_FLAG.fetch_not(Ordering::Relaxed);
    syskrnl::gui::status_bar::update_status_bar_time(eof);
    if !eof {
        // debugln!("Half sec: {:?}", GUI_TIME_UPDATE_EVENT_NEEDER.lock());
        for eid in GUI_TIME_UPDATE_EVENT_NEEDER.lock().iter() {
            if let Some(pid) = EVENT_QUEUE.lock().wakeup_with_ret(*eid, gui_event_make_ret(GUI_EVENT_TIME_UPDATE, 0, 0, 0)) {
                SCHEDULER.lock().wakeup(pid);
            }
        }
    }
}

