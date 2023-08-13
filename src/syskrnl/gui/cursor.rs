use alloc::vec::Vec;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use spin::{Mutex, RwLock};

use crate::syskrnl::graphic::{GL, HEIGHT, resolve_32rgba, WIDTH};
use crate::syskrnl::gui::WINDOW_MANAGER;

const CURSOR: &[u8] = include_bytes!("../../../assets/cursor.bmp");
lazy_static! {
    static ref RESOLVED_CURSOR:RwLock<Vec<(usize, usize, Rgb888)>> = {
      RwLock::new(resolve_32rgba(CURSOR))
    };
}
const ACCELERATE: f32 = 2.0;

lazy_static! {
    pub static ref MOUSE_CURSOR: Mutex<MouseCursor> = {
        Mutex::new(MouseCursor::new(HEIGHT as i32, WIDTH as i32, (HEIGHT as i32)/2, (WIDTH as i32)/2))
    };
}

pub struct MouseCursor {
    max_x: i32,
    max_y: i32,
    x: i32,
    y: i32,
    last_x: i32,
    last_y: i32,
}

impl MouseCursor {
    pub fn new(max_x: i32, max_y: i32, x: i32, y: i32) -> Self {
        Self { max_x, max_y, x, y, last_x: x, last_y: y }
    }
    pub fn print_manually(&self) {
        let pos = GL.read().len() - 1;
        GL.read()[pos].lock().display_resolved(self.x as usize, self.y as usize, &RESOLVED_CURSOR.read());
    }
    pub fn handle_change(&mut self, dy: i32, dx: i32) {
        let nx = self.x + (dx as f32 * ACCELERATE) as i32;
        let ny = self.y + (dy as f32 * ACCELERATE) as i32;
        self.x = if nx < 0 { 0 } else if nx > self.max_x { self.max_x } else { nx };
        self.y = if ny < 0 { 0 } else if ny > self.max_y { self.max_y } else { ny };
    }
    pub fn update_mouse(&mut self) {
        let dx = self.x - self.last_x;
        let dy = self.y - self.last_y;
        if dx != 0 || dy != 0 {
            let pos = GL.read().len() - 1;
            // debugln!("Prepare to move: {}, {}",dx, dy);
            GL.read()[pos].lock().clear_resolved(self.last_x as usize, self.last_y as usize, &RESOLVED_CURSOR.read());
            GL.read()[pos].lock().display_resolved(self.x as usize, self.y as usize, &RESOLVED_CURSOR.read());
            self.last_x = self.x;
            self.last_y = self.y;
        }
    }
    pub fn handle_click(&mut self) {
        WINDOW_MANAGER.lock().handle_mouse_click(self.x as usize, self.y as usize);
    }
}