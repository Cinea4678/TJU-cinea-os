use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Deref;
use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use spin::{Mutex, RwLock};
use cinea_os_sysapi::fs::read_all_from_path;

pub use cinea_os_sysapi::window::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::rgb888;
use crate::syskrnl::graphic::{GL, resolve_32rgba};
use crate::syskrnl::{graphic, proc};

lazy_static! {
    pub static ref WINDOW_MANAGER: Mutex<WindowManager> = Mutex::new(WindowManager::new());
    static ref ASSETS: RwLock<BTreeMap<String,Vec<(usize, usize, Rgb888)>>> = RwLock::new(BTreeMap::new());
}

pub struct WindowManager {
    windows: Vec<Window>,
    layout: WindowLayoutManager,
    highlight: usize
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
                (30, 450, false),
                (250, 450, false),
            ]),
            highlight: 0
        }
    }

    pub fn create_window(&mut self, title: &str, gm_addr: usize) -> bool {
        let pid = proc::id();
        if let None = self.windows.iter().find(|w| w.process_id == pid) {
            if self.windows.len() < self.layout.layouts.len() {
                self.windows.push(Window::new(pid, title, gm_addr));
                self.layout.layouts[self.windows.len() - 1].2 = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn draw_window_frame(&self, x: usize, y: usize, window: &Window, writer: &mut graphic::Writer) {
        writer.display_rect(x, y, WINDOW_WIDTH, 20, rgb888!(0x262A10u32));
        writer.display_resolved(x + 2, y + 2, ASSETS.read().get("WindowCloseBtn").expect("Read ASSETS Fail"));
        unsafe { writer.display_font_string(window.title.as_str(), x + 2, y + 20, 16.0, 16, rgb888!(0xFFFFFFu32)) };
        writer.display_rect(x + 20, y, 2, WINDOW_HEIGHT - 20, rgb888!(0x262A10u32));
        writer.display_rect(x + 20, y + WINDOW_WIDTH - 2, 2, WINDOW_HEIGHT - 20, rgb888!(0x262A10u32));
        writer.display_rect(x + WINDOW_HEIGHT - 2, y, WINDOW_WIDTH, 2, rgb888!(0x262A10u32));
    }

    pub fn render(&mut self) {
        let p_lock = GL.read();
        let mut lock = p_lock[2].lock();
        for (i, window) in self.windows.iter().enumerate() {
            if i == self.highlight {
                continue;
            }
            if self.layout.layouts[i].2 {
                let layout = &self.layout.layouts[i];
                self.draw_window_frame(layout.0, layout.1, window, &mut lock);
                let data = unsafe {
                    &*(window.mem_addr as *const Vec<Vec<Rgb888>>)
                };
                lock.display_from_copied(layout.0 + 20, layout.1 + 2, data);
            }
        }
        if self.windows.len() > 0 {
            if self.layout.layouts[self.highlight].2 {
                let layout = &self.layout.layouts[self.highlight];
                let window = &self.windows[self.highlight];
                self.draw_window_frame(layout.0, layout.1, window, &mut lock);
                let data = unsafe {
                    &*(window.mem_addr as *const Vec<Vec<Rgb888>>)
                };
                lock.display_from_copied(layout.0 + 20, layout.1 + 2, data);
            }
        }
    }
}

pub fn init() {
    let close_btn = read_all_from_path(String::from("/sys/ast/window_close_btn.bmp")).expect("Read ASSETS fail");
    ASSETS.write().insert(String::from("WindowCloseBtn"), resolve_32rgba(close_btn.as_slice()));
}
