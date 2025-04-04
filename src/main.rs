use macroquad::prelude::*;

mod emulator;

#[macroquad::main("CHIP-8")]
async fn main() {
    loop {
        clear_background(BLACK);
        next_frame().await
    }
}