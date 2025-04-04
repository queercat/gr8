use std::{fs, io};

use super::opcode::Opcode;

#[derive(Debug)]
struct Emulator {
    memory: [i8; 4096],
    registers: [i8; 16],
    address: i16,
    display: [[i8; 64]; 32],
    delay_timer: i8,
    sound_timer: i8,
    input: [i8; 16],
    stack: [i8; 48],
    instructions: Vec<Opcode>,
}

impl Emulator {
    fn new() -> Self {
        Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            address: 0,
            display: [[0; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            input: [0; 16],
            stack: [0; 48],
            instructions: Vec::new(),
        }
    }

    pub fn load_rom(&mut self, path_to_rom: &str) -> Result<(), &'static str> {
        let rom_data = match fs::read(path_to_rom) {
            Ok(rom_data) => rom_data,
            Err(_) => return Err("Failed to read ROM.")
        };

        match Opcode::decode(&rom_data) {
            Ok(instructions) => self.instructions = instructions,
            Err(e) => return Err(e)
        };

        Ok(())
    }

    /// The core update loop for the emulator.
    fn tick(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct_emulator() {
        let _emulator = Emulator::new();
    }

    #[test]
    fn can_load__rom() {
        let mut emulator = Emulator::new();

        let result = emulator.load_rom("./src/examples/chip8-roms/demos/Particle Demo [zeroZshadow, 2008].ch8");

        assert!(result.is_ok())
    }
}
