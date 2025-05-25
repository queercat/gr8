mod emulator;

use macroquad::prelude::*;
use emulator::Emulator;


#[macroquad::main("GR8")]
async fn main() {
    let mut emulator = Emulator::new();
    emulator.load_rom("src/examples/chip8-roms/programs/Chip8 Picture.ch8").unwrap();

    loop {
        emulator.update();
        clear_background(BLACK);
        next_frame().await
    }
}
