mod emulator;

use macroquad::prelude::*;
use emulator::Emulator;

#[macroquad::main("GR8")]
async fn main() {
    let mut emulator = Emulator::new();
    emulator.load_rom("src/examples/chip8-roms/programs/IBM Logo.ch8").unwrap();

    let width = screen_width() as i32;
    let height = screen_height() as i32;
    let dx = width / 64;
    let dy = height / 32;

    loop {
        clear_background(BLACK);

        emulator.update();

        for y in 0..32 {
            for x in 0..64 {
                let color = match x % 4 {
                    _ => WHITE
                };

                if emulator.display[y as usize][x as usize] == 0 { continue; }

                draw_rectangle(
                    (x * dx) as f32, 
                    (y * dy) as f32, 
                    dx as f32, 
                    dy as f32,
                    color);
            }
        }

        next_frame().await
    }
}
