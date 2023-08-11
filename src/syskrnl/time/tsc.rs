use core::{arch, hint};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::debugln;

pub static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

pub fn rdtsc() -> u64 {
    unsafe {
        arch::x86_64::_mm_lfence();  // 确保读取到的是最新的值
        arch::x86_64::_rdtsc()
    }
}

pub fn nanowait(nanoseconds: u64) {
    let start = rdtsc();
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
    while rdtsc() - start < delta {
        hint::spin_loop(); // 短途睡眠
    }
}

/// 初始化并记录TSC每纳秒的递增数
pub fn init(){
    let calibration_time = 250_000;
    // 0.25 seconds
    let a = rdtsc();
    super::sleep(calibration_time as f64 / 1e6);
    let b = rdtsc();
    CLOCKS_PER_NANOSECOND.store((b - a) / calibration_time, Ordering::Relaxed);
}
