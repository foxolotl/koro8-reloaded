use sdl2::{image::LoadTexture};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let rom_path = args.first().expect("no arguments given");
    let rom = std::fs::read(rom_path).expect(&format!("no such file: {}", rom_path));

    let sdl = sdl2::init().unwrap();
    let canvas = koro8::peripherals::sdl::display::Display::create_canvas(&sdl).unwrap();
    let texture_creator = canvas.texture_creator();
    let pixel_texture = texture_creator.load_texture("graphics/pixel.png").unwrap();
    let background_texture = texture_creator.load_texture("graphics/background.png").unwrap();
    let mut display = koro8::peripherals::sdl::display::Display::new(
        canvas,
        pixel_texture,
        background_texture
    ).unwrap();

    let rng = rand::rngs::OsRng;
    let keyboard = koro8::peripherals::sdl::keyboard::Keyboard::new(&sdl).unwrap();
    let buzzer = koro8::peripherals::sdl::buzzer::Buzzer::new(
        &sdl,
        "sounds",
        4,
        Box::new(rng)
    ).unwrap();
    let mut cpu = koro8::cpu::new(
        &mut display,
        Box::new(keyboard),
        Box::new(buzzer),
        Box::new(rng),
        11
    );
    cpu.load(&rom);
    cpu.run();
    drop(cpu);
}
