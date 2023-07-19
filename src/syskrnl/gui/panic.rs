use crate::{hlt_loop, rgb888};
use crate::syskrnl::graphic::{GD, GL};
use crate::syskrnl::graphic::text::TextWriter;

pub struct PanicInfo<'a> {
    title: &'a str,
    description: &'a str,
}

impl<'a> PanicInfo<'a> {
    pub fn new(title: &'a str, description: &'a str) -> PanicInfo<'a> {
        PanicInfo {
            title,
            description,
        }
    }

    pub fn get_title(&self) -> &str {
        self.title
    }

    pub fn get_description(&self) -> &str {
        self.description
    }
}

pub fn handle_panic(info: &PanicInfo) -> ! {
    let p_lock = GL.read();
    for layer in 0..2 {
        p_lock[layer].lock().enable = true;
    }
    for layer in 2..p_lock.len() {
        p_lock[layer].lock().enable = false;
    }

    let mut lock = p_lock[0].lock();
    lock.display_img(0,0,include_bytes!("../../../assets/BlueScreen_small.bmp"));
    drop(lock);
    let p_lock = GL.read();
    let mut lock = p_lock[1].lock();
    for r in lock.data.iter_mut(){
        for p in r.iter_mut(){
            p.0 = rgb888!(0);
            p.1 = false;
        }
    }
    drop(lock);

    let mut writer = TextWriter{
        text_area_height: 570,
        text_area_width: 700,
        text_area_pos: (22, 40),
        y_position: 0,
        line_position: 7,
        line_height: 16,
        line_gap: 4,
        max_line: 28,
        color: rgb888!(0xffffffu32),
        layer: 1,
    };

    writer.write_string("以下是有关错误的信息");
    writer.write_string("\n\n");
    writer.write_string(info.get_title());
    writer.write_string("\n");
    writer.write_string(info.get_description());

    GD.lock().render(0,0,600,800);

    hlt_loop();
}