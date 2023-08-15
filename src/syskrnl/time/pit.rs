use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use x86_64::instructions::interrupts;
use x86_64::instructions::interrupts::without_interrupts;

use crate::syskrnl;
use crate::syskrnl::graphic::GD;
use crate::syskrnl::gui::cursor::MOUSE_CURSOR;
use crate::syskrnl::gui::{RENDER_OK, WINDOW_MANAGER};

/// `PIT_FREQUENCY`的值是x86架构默认的
pub const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // 1_193_181.666 Hz
/// `PIT_DIVIDER`的值算个半魔数，用来得到大概1ms的PIT频率
const PIT_DIVIDER: usize = 1193;
/// 两次PIT中断之间的时间间隔
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;
/// 每秒Tick数
pub const PIT_PER_SECOND: usize = 1000;

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
static RENDER: AtomicU8 = AtomicU8::new(0);

/// 设置PIT频率分频器
///
/// `divider` - 分频器
/// `channel` - 通道（听说有4个?）
pub fn set_pit_frequency_divider(divider: u16, channel: u8) {
    // 关中断
    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let operating_mode = 6; // 方波生成器
        let access_mode = 3; // 低字节和高字节都要写入
        unsafe {
            // 选择通道和模式
            x86::io::outb(0x43, (channel << 6) | (access_mode << 4) | operating_mode);
            // 写入分频器
            x86::io::outb(0x40, bytes[0]);
            x86::io::outb(0x40, bytes[1]);
        }
    })
}

/// PIT中断处理程序
pub fn pit_interrupt_handler() {
    let time = PIT_TICKS.fetch_add(1, Ordering::Relaxed);

    // 每1/25秒渲染一次
    if RENDER_OK.load(Ordering::Relaxed) && time % 40 == 0 {
        RENDER.store(7, Ordering::Relaxed);
    }

    let mut render_flag = RENDER.load(Ordering::Relaxed);

    if render_flag > 0 {
        without_interrupts(|| {
            if let Some(mut mouse_lock) = MOUSE_CURSOR.try_lock() {
                mouse_lock.update_mouse();
                render_flag &= !4;
            }
            if let Some(mut window_lock) = WINDOW_MANAGER.try_lock() {
                window_lock.render();
                render_flag &= !2;
            }
            if let Some(mut gd_lock) = GD.try_lock() {
                gd_lock.render(0, 0, 600, 800);
                render_flag &= !1;
            }
        });
        RENDER.store(render_flag, Ordering::Relaxed);
    }
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

pub fn init() {
    // PIT计时器
    set_pit_frequency_divider(PIT_DIVIDER as u16, 0);
    syskrnl::interrupts::set_irq_handler(0, pit_interrupt_handler);
    // debugln!("{},{}",PIT_FREQUENCY,PIT_INTERVAL);
}
