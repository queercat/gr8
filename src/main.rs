mod emulator;

use macroquad::prelude::*;
use emulator::Emulator;
use std::time::SystemTime;

#[macroquad::main("GR8")]
async fn main() {
    let mut emulator = Emulator::new();
    emulator.load_rom("src/examples/chip8-roms/programs/Clock Program [Bill Fisher, 1981].ch8").unwrap();

    let time = SystemTime::now();


    loop {
        let width = screen_width() as i32;
        let height = screen_height() as i32;
        let dx = width / 64;
        let dy = height / 32;

        clear_background(BLACK);

        emulator.time_in_ms = time.elapsed().expect("I am genuinely uncertain as to why this would happen.").as_millis();

        emulator.update().expect("Couldn't update");

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
