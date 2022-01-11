# koro8 reloaded
A [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator with
[Korone](https://twitter.com/KoroneNoises/)-based sounds and visuals.

Because the original [koro8](https://github.com/foxolotl/koro8) wasn't Korone enough.

## Usage
0. Make sure you have Rust, Cargo, SDL, SDL_Mixer and SDL_Image installed.
1. Get some [roms](https://github.com/loktar00/chip8/tree/master/roms) to run.    

    [Breakout](https://github.com/loktar00/chip8/raw/master/roms/Breakout%20%5BCarmelo%20Cortez%2C%201979%5D.ch8)
    is a great choice for experiencing the cutting-edge doggo sound chip of koro8.
    Using the default key map, the A and D keys control the paddle.
2. Check out the repository and run `cargo run /path/to/some/rom.ch8`.

## Controls
The CHIP-8 input consists of 16 keys, numbered from 0 to F.
The koro8 key map is QWER to 123C, ASDF to 456D, ZXCV to 789E, and 1234 to A0BF.
Most roms use keys 2, 4, 6 and 8 for directional input, which map to W, A, D and X respectively.

To reset the loaded rom, press ESC. To quit, just close the window.
