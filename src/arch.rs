pub const NUM_KEYS: usize = 16;
pub const SCALE: usize = 16;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const PIXELS: usize = WIDTH as usize * HEIGHT as usize;

pub struct Sprite<'a>(pub &'a [u8]);

pub trait Display {
    fn clear(&mut self);
    fn draw(&mut self, sprite: &Sprite, x: u8, y: u8) -> bool;
    fn present(&mut self);
    fn reset(&mut self);
}

pub trait Keyboard {
    fn pressed(&mut self, key: u8) -> bool;
    fn wait_key(&mut self) -> u8;
    fn reset_signal(&mut self) -> bool;
    fn power_off_signal(&mut self) -> bool;
    fn reset(&mut self);
}

pub trait Buzzer {
    fn start(&mut self);
    fn stop(&mut self);
    fn reset(&mut self);
}
