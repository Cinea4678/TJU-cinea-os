use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;

#[macro_export]
macro_rules! rgb888 {
    ($num:expr) => {
        embedded_graphics::pixelcolor::Rgb888::new(($num>>16) as u8,($num>>8) as u8,$num as u8)
    };
}

pub fn alpha_mix(fg:Rgb888, fga: f32, bg:Rgb888, bga: f32)->(Rgb888,f32){
    let a = fga +bga * (1.0 - fga);
    let r = (fg.r() as f32 * fga + bg.r() as f32 * bga * (1.0 - fga)) / a;
    let g = (fg.g() as f32 * fga + bg.g() as f32 * bga * (1.0 - fga)) / a;
    let b = (fg.b() as f32 * fga + bg.b() as f32 * bga * (1.0 - fga)) / a;

    (Rgb888::new(r as u8,g as u8, b as u8), a)
}

pub fn alpha_mix_final(fg:Rgb888, fga: f32, bg:Rgb888)->Rgb888{
    let r = (fg.r() as f32 * fga + bg.r() as f32 * (1.0 - fga)) / a;
    let g = (fg.g() as f32 * fga + bg.g() as f32 * (1.0 - fga)) / a;
    let b = (fg.b() as f32 * fga + bg.b() as f32 * (1.0 - fga)) / a;

    Rgb888::new(r as u8,g as u8, b as u8)
}
