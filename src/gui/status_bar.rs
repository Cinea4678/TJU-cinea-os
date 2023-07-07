use crate::graphic::{GD, WIDTH};
use crate::rgb888;

fn show_status_bar(){
    GD.lock().display_rect(0,0,WIDTH, 20, rgb888!(0x0000aa));

}