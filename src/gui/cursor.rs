use crate::graphic::{GD, GL};

const CURSOR: &[u8] = include_bytes!("../../assets/cursor.bmp");

pub fn display_cursor_first_time(x: usize, y: usize) {
    let lock = GL.lock();
    lock[lock.len() - 1].display_img(x, y, CURSOR);
}