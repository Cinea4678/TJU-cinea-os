use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use spin::{Mutex, RwLock};

use cinea_os_sysapi::event::{gui_event_make_ret, GUI_EVENT_EXIT, GUI_EVENT_MOUSE_CLICK};
use cinea_os_sysapi::fs::read_all_from_path;
use cinea_os_sysapi::gui::WindowGraphicMemory;
pub use cinea_os_sysapi::gui::{WINDOW_HEIGHT, WINDOW_WIDTH};

use crate::rgb888;
use crate::syskrnl::event::{EVENT_QUEUE, GUI_EID_START};
use crate::syskrnl::graphic::{resolve_32rgba, GL, HEIGHT, WIDTH};
use crate::syskrnl::proc::SCHEDULER;
use crate::syskrnl::{graphic, proc};

lazy_static! {
    pub static ref WINDOW_MANAGER: Mutex<WindowManager> = Mutex::new(WindowManager::new());
    static ref ASSETS: RwLock<BTreeMap<String, Vec<(usize, usize, Rgb888)>>> = RwLock::new(BTreeMap::new());
}

pub struct WindowManager {
    windows: Vec<Window>,
    layout: WindowLayoutManager,
    highlight: usize,
    moving_window_now: bool,
}

pub struct WindowLayoutManager {
    /// 起始x，起始y，是否使用
    layouts: Vec<(usize, usize, bool)>,
}

pub struct Window {
    process_id: usize,
    title: String,
    mem_addr: usize,
}

impl WindowLayoutManager {
    pub fn new() -> Self {
        Self { layouts: Vec::new() }
    }

    pub fn new_from_layout(layouts: Vec<(usize, usize, bool)>) -> Self {
        Self { layouts }
    }

    pub fn in_window(&self, index: usize, x: usize, y: usize) -> bool {
        if let Some(layout) = self.layouts.get(index) && layout.2 {
            layout.0 <= x && x < layout.0 + WINDOW_HEIGHT && layout.1 <= y && y < layout.1 + WINDOW_WIDTH
        } else {
            false
        }
    }
}

impl Window {
    pub fn new(process_id: usize, title: &str, mem_addr: usize) -> Self {
        Self {
            process_id,
            title: String::from(title),
            mem_addr,
        }
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            layout: WindowLayoutManager::new_from_layout(vec![(30, 450, false), (250, 450, false)]),
            highlight: 0,
            moving_window_now: false,
        }
    }

    pub fn create_window(&mut self, title: &str, gm_addr: usize) -> bool {
        let pid = proc::id();
        if let None = self.windows.iter().find(|w| w.process_id == pid) {
            if self.windows.len() < self.layout.layouts.len() {
                let layout_i = self.layout.layouts.iter().enumerate().find(|l| l.1 .2 == false).unwrap().0;
                if layout_i >= self.windows.len() {
                    self.windows.push(Window::new(pid, title, gm_addr));
                } else {
                    self.windows[layout_i] = Window::new(pid, title, gm_addr);
                }
                self.layout.layouts[layout_i].2 = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn destory_window(&mut self) {
        let pid = proc::id();
        if let Some((i, _window)) = self.windows.iter().enumerate().find(|w| w.1.process_id == pid) {
            self.layout.layouts[i].2 = false;
            EVENT_QUEUE.lock().switch_front(1);
        }
    }

    fn draw_window_frame(&self, x: usize, y: usize, window: &Window, writer: &mut graphic::Writer, active: bool) {
        let main_color = if active { rgb888!(0x262A10u32) } else { rgb888!(0x54442Bu32) };
        writer.display_rect(x, y, WINDOW_WIDTH, 20, main_color);
        writer.display_resolved(x + 2, y + 2, ASSETS.read().get("WindowCloseBtn").expect("Read ASSETS Fail"));
        writer.display_resolved(x + 2, y + 20, ASSETS.read().get("WindowMoveBtn").expect("Read ASSETS Fail"));
        unsafe { writer.display_font_string(window.title.as_str(), x + 2, y + 40, 16.0, 16, rgb888!(0xFFFFFFu32)) };
        writer.display_rect(x + 20, y, 2, WINDOW_HEIGHT - 20, main_color);
        writer.display_rect(x + 20, y + WINDOW_WIDTH - 2, 2, WINDOW_HEIGHT - 20, main_color);
        writer.display_rect(x + WINDOW_HEIGHT - 2, y, WINDOW_WIDTH, 2, main_color);
        if active {
            writer.display_resolved(x + 2, y + WINDOW_WIDTH - 20, ASSETS.read().get("WindowActive").expect("Read ASSETS Fail"));
        } else {
            writer.display_resolved(
                x + 2,
                y + WINDOW_WIDTH - 20,
                ASSETS.read().get("WindowInactive").expect("Read ASSETS Fail"),
            );
        }
    }

    pub fn render(&mut self) {
        let p_lock = GL.read();
        let mut lock = p_lock[2].lock();
        lock.clear_rect(0, 0, WIDTH, HEIGHT);
        for (i, window) in self.windows.iter().enumerate() {
            if i == self.highlight {
                continue;
            }
            if self.layout.layouts[i].2 {
                let layout = &self.layout.layouts[i];
                self.draw_window_frame(layout.0, layout.1, window, &mut lock, false);
                let data = unsafe { &*(window.mem_addr as *const WindowGraphicMemory) };
                lock.display_from_copied(layout.0 + 20, layout.1 + 2, data);
            }
        }
        if self.windows.len() > 0 {
            if self.layout.layouts[self.highlight].2 {
                let layout = &self.layout.layouts[self.highlight];
                let window = &self.windows[self.highlight];
                self.draw_window_frame(layout.0, layout.1, window, &mut lock, true);
                let data = unsafe { &*(window.mem_addr as *const WindowGraphicMemory) };
                lock.display_from_copied(layout.0 + 20, layout.1 + 2, data);
            }
        }
    }

    fn window_handle_click(&mut self, window_index: usize, x: usize, y: usize) {
        let x = x - self.layout.layouts[window_index].0;
        let y = y - self.layout.layouts[window_index].1;
        // debugln!("Window Handle Click: {} {} {}",window_index,x,y);

        if x < 20 {
            if y < 18 {
                // Close Button
                let ret = gui_event_make_ret(GUI_EVENT_EXIT, 0, 0, 0);
                if let Some(pid) = EVENT_QUEUE
                    .lock()
                    .wakeup_with_ret(GUI_EID_START + self.windows[window_index].process_id, ret)
                {
                    SCHEDULER.lock().wakeup(pid);
                }
            } else if y < 38 {
                // Move Button
                self.moving_window_now = true
            }
        } else {
            let x = x - 20;
            let y = y - 2;
            let ret = gui_event_make_ret(GUI_EVENT_MOUSE_CLICK, x as u16, y as u16, 0);
            if let Some(pid) = EVENT_QUEUE
                .lock()
                .wakeup_with_ret(GUI_EID_START + self.windows[window_index].process_id, ret)
            {
                SCHEDULER.lock().wakeup(pid);
            }
        }
    }

    pub fn handle_mouse_click(&mut self, x: usize, y: usize) {
        //
        // 处理逻辑简述：
        //
        // 0. 确保当前没有正在移动窗口
        // 1. 首先检查是否在高亮窗口区域内
        // 2. 其次按照倒序查找窗口（避免重叠的选错）
        // 3. 如果在窗口内：
        //     活动窗口：检查是否正在点击按钮（关闭、移动）
        //     非活动窗口：转为活动窗口
        //

        debugln!("Mouse Click:{},{}", x, y);

        if self.moving_window_now {
            // 移动窗口到鼠标位置
            let layout = &mut self.layout.layouts[self.highlight];
            layout.0 = x;
            layout.1 = y;
            self.moving_window_now = false;
        } else if self.layout.in_window(self.highlight, x, y) {
            self.window_handle_click(self.highlight, x, y);
        } else {
            for (i, window) in self.windows.iter().enumerate().rev() {
                if i != self.highlight {
                    if self.layout.in_window(i, x, y) {
                        self.highlight = i;
                        EVENT_QUEUE.lock().switch_front(window.process_id);
                        break;
                    }
                }
            }
            EVENT_QUEUE.lock().switch_front(1); // 都不是活动的，那就是shell了。
        }
    }
}

pub fn init() {
    let close_btn = read_all_from_path("/sys/ast/window_close_btn.bmp").expect("Read ASSETS fail");
    let move_btn = read_all_from_path("/sys/ast/window_move_btn.bmp").expect("Read ASSETS fail");
    let active = read_all_from_path("/sys/ast/window_active.bmp").expect("Read ASSETS fail");
    let inactive = read_all_from_path("/sys/ast/window_inactive.bmp").expect("Read ASSETS fail");
    ASSETS
        .write()
        .insert(String::from("WindowCloseBtn"), resolve_32rgba(close_btn.as_slice()));
    ASSETS.write().insert(String::from("WindowMoveBtn"), resolve_32rgba(move_btn.as_slice()));
    ASSETS.write().insert(String::from("WindowActive"), resolve_32rgba(active.as_slice()));
    ASSETS.write().insert(String::from("WindowInactive"), resolve_32rgba(inactive.as_slice()));
}
