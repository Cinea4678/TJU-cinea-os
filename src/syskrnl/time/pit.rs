use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::instructions::interrupts;
use crate::{debug, debugln, syskrnl};

/// `PIT_FREQUENCY`的值是x86架构默认的
pub const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // 1_193_181.666 Hz
/// `PIT_DIVIDER`的值算个半魔数，用来得到大概1ms的PIT频率
const PIT_DIVIDER: usize = 1193;
/// 两次PIT中断之间的时间间隔
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

/// 设置PIT频率分频器
///
/// `divider` - 分频器
/// `channel` - 通道（听说有4个?）
pub fn set_pit_frequency_divider(divider: u16, channel: u8){
    // 关中断
    interrupts::without_interrupts(||{
        let bytes = divider.to_le_bytes();
        let operating_mode = 6; // 方波生成器
        let access_mode = 3; // 低字节和高字节都要写入
        unsafe {
            // 选择通道和模式
            x86::io::outb(0x43,(channel << 6) | (access_mode << 4) | operating_mode);
            // 写入分频器
            x86::io::outb(0x40,bytes[0]);
            x86::io::outb(0x40,bytes[1]);
        }
    })
}

/// PIT中断处理程序
pub fn pit_interrupt_handler(){
    PIT_TICKS.fetch_add(1,Ordering::Relaxed);
}

pub fn get_ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn get_uptime() -> f64 {
    (get_ticks() as f64) * PIT_INTERVAL
}

pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

pub fn init(){
    // PIT计时器
    set_pit_frequency_divider(PIT_DIVIDER as u16, 0);
    syskrnl::interrupts::set_irq_handler(0, pit_interrupt_handler);
    debugln!("{},{}",PIT_FREQUENCY,PIT_INTERVAL);
}