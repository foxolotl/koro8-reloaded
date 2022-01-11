use sdl2::{render::{Canvas, Texture}, video::Window, Sdl, rect::Rect};

use crate::arch::{PIXELS, WIDTH, SCALE, HEIGHT};

pub struct Display<'a> {
    canvas: Canvas<Window>,
    pixel: Texture<'a>,
    background: Texture<'a>,
    pixels: [bool;PIXELS]
}

impl <'a> Display<'a> {
    pub fn create_canvas(sdl: &Sdl) -> Option<Canvas<Window>> {
        let video  = sdl.video().ok()?;
        let window = video
            .window("koro8", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
            .position_centered()
            .build()
            .ok()?;
        window.into_canvas().accelerated().build().ok()
    }

    pub fn new(
        canvas: Canvas<Window>,
        pixel: Texture<'a>,
        background: Texture<'a>
    ) -> Option<Display<'a>> {
        let display = Display {
            canvas,
            pixel,
            background,
            pixels: [false;PIXELS]
        };
        Some(display)
    }
}

impl <'a> crate::arch::Display for Display<'a> {
    fn clear(&mut self) {
        self.pixels = [false;PIXELS];
    }

    fn draw(&mut self, sprite: &crate::arch::Sprite, x: u8, y: u8) -> bool {
        let mut pixel_changed = false;
        for (row, row_ix) in sprite.0.iter().zip(0..sprite.0.len()) {
            for col_ix in 0..8 {
                let py = (row_ix + y as usize) % HEIGHT;
                let px = (col_ix + x as usize) % WIDTH;
                let ix = py * WIDTH + px;
                let old_pixel = self.pixels[ix];
                let new_pixel = ((row >> (7 - col_ix)) & 1) == 1;
                self.pixels[ix] = old_pixel ^ new_pixel;
                if old_pixel && new_pixel {
                    pixel_changed = true;
                }
            }
        }
        self.present();
        pixel_changed
    }

    fn present(&mut self) {

        let pixels = &self.pixels;
        let pixel = &self.pixel;
        let canvas = &mut self.canvas;
        let src_rect = Rect::new(0, 0, 16, 16);
        canvas.copy(
            &self.background,
            Rect::new(0, 0, (WIDTH*SCALE) as u32, (HEIGHT*SCALE) as u32),
            Rect::new(0, 0, (WIDTH*SCALE) as u32, (HEIGHT*SCALE) as u32)
        ).unwrap();
        (0..HEIGHT).for_each(|y| {
            (0..WIDTH).for_each(|x| {
                if pixels[y * WIDTH + x] {
                    let dst_rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE as u32, SCALE as u32);
                    canvas.copy(pixel, src_rect, dst_rect).unwrap();
                }
            })
        });
        self.canvas.present();
    }

    fn reset(&mut self) {
        self.pixels = [false;PIXELS];
    }
}
