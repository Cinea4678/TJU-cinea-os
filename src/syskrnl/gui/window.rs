use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

pub use cinea_os_sysapi::window::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::syskrnl::proc;

lazy_static! {
    pub static ref WINDOW_MANAGER: Mutex<WindowManager> = Mutex::new(WindowManager::new());
}

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

impl WindowLayoutManager {
    pub fn new() -> Self {
        Self { layouts: Vec::new() }
    }

    pub fn new_from_layout(layouts: Vec<(usize, usize, bool)>) -> Self {
        Self { layouts }
    }
}

impl Window {
    pub fn new(process_id: usize, title: &str, mem_addr: usize) -> Self {
        Self { process_id, title: String::from(title), mem_addr }
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            layout: WindowLayoutManager::new_from_layout(vec![
                (600, 0, false),
                (600, 200, false),
                (600, 400, false),
                (400, 400, false),
                (400, 200, false),
                (400, 0, false),
            ]),
        }
    }
    pub fn create_window(&mut self, title: &str, gm_addr: usize) -> bool {
        let pid = proc::id();
        if let None = self.windows.iter().find(|w| w.process_id == pid) {
            if self.windows.len() < self.layout.layouts.len() {
                self.windows.push(Window::new(pid, title, gm_addr));
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
