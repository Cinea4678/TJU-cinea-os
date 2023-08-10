use alloc::string::String;
use alloc::vec::Vec;

pub use cinea_os_sysapi::window::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct WindowManager {
    windows: Vec<Window>,
    layout: WindowLayoutManager,
}

pub struct WindowLayoutManager {
    /// 起始x，起始y，是否使用
    layouts: Vec<(usize, usize, bool)>,
}

pub struct Window {
    process_id: usize,
    title: String,
    mem_addr: usize
}

