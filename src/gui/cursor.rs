use crate::graphic::{GL};

const CURSOR: &[u8] = include_bytes!("../../assets/cursor.bmp");

pub fn display_cursor_first_time(x: usize, y: usize) {
    let pos = GL.read().len() - 1;
    GL.read()[pos].lock().display_img_32rgba(x, y, CURSOR);
}