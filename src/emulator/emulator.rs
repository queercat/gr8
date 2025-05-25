use super::opcode::Opcode;
use crate::emulator::opcode::ToBits;
use rand::random_range;
use std::fs;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const MEMORY_SIZE: usize = 4096;
pub const REGISTER_COUNT: usize = 16;
pub const STACK_SIZE: usize = 48;
pub const FONT_DATA_ADDRESS: usize = 0x20;

#[derive(Debug)]
pub struct Emulator {
    pub display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT],
    address: u16,
    delay_timer: u8,
    sound_timer: u8,
    input: [u8; 16],
    stack: [u8; 48],
    sp: usize,
    pc: usize,
    awaiting_keypress: bool,
}

impl From<Vec<Opcode>> for Emulator {
    fn from(opcodes: Vec<Opcode>) -> Self {
        let mut emulator = Emulator::new();
        emulator.load_instructions(opcodes.to_bits());
        emulator
    }
}

#[derive(Debug, PartialEq)]
pub enum EmulatorStatus {
    Working,
    Done,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Emulator {
            memory: [0; MEMORY_SIZE],
            registers: [0; REGISTER_COUNT],
            address: 0,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            input: [0; 16],
            stack: [0; STACK_SIZE],
            sp: 0,
            pc: 0x200,
            awaiting_keypress: false,
        };

        emulator.init();

        emulator
    }

    fn init(&mut self) {
        let font_data: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
            [0x20, 0x60, 0x20, 0x20, 0x70], // 1
            [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
            [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
            [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
            [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
            [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
            [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
            [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
            [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
            [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
            [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
            [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
            [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
            [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
            [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
        ];

        for letter_idx in 0..font_data.len() {
            for byte_idx in 0..font_data[letter_idx].len() {
                self.memory[FONT_DATA_ADDRESS + letter_idx * 5 + byte_idx] =
                    font_data[letter_idx][byte_idx];
            }
        }
    }

    fn with_opcodes(mut self, opcodes: Vec<Opcode>) -> Self {
        self.load_instructions(opcodes.to_bits()).unwrap();
        self
    }

    fn with_register_as(mut self, r: u8, v: u8) -> Self {
        self.registers[r as usize] = v;
        self
    }

    fn with_input_as(mut self, r: u8, v: u8) -> Self {
        self.input[r as usize] = v;
        self
    }

    fn with_display(mut self, display: [[u8; 64]; 32]) -> Self {
        self.display = display;
        self
    }

    fn load_instructions(&mut self, instructions: Vec<u8>) -> Result<(), String> {
        dbg!(&instructions.len());
        for i in 0..instructions.len() {
            self.memory[i + 0x200] = instructions[i];
        }
        Ok(())
    }

    pub fn load_rom(&mut self, path_to_rom: &str) -> Result<(), String> {
        let rom_data = fs::read(path_to_rom).map_err(|e| e.to_string())?;
        self.load_instructions(rom_data);

        Ok(())
    }

    fn goto(&mut self, address: u16) {
        self.pc = address as usize;
    }

    fn call_subroutine(&mut self, address: u16) -> Result<(), String> {
        if self.sp >= self.stack.len() {
            return Err("Stack overflow!".to_string());
        }

        self.stack[self.sp] = self.pc as u8;
        self.sp += 2;

        self.goto(address);

        Ok(())
    }

    fn r#return(&mut self) -> Result<(), String> {
        if self.sp == 0 {
            return Err("Not in a subroutine!".to_string());
        }

        self.goto(self.stack[self.sp] as u16);
        self.sp -= 1;

        Ok(())
    }

    fn fetch_and_decode(&mut self) -> Result<Opcode, String> {
        let instruction = (self.memory[self.pc], self.memory[self.pc + 1]);
        self.pc += 2;
        Ok(Opcode::decode(instruction)?)
    }

    pub fn update(&mut self) -> Result<EmulatorStatus, String> {
        let opcode = self.fetch_and_decode()?;

        dbg!(&opcode);

        match opcode {
            Opcode::ClearScreen => {
                let this = &mut *self;
                this.display = [[0; 64]; 32]
            }
            Opcode::Goto(address) => self.goto(address),
            Opcode::CallSubroutine(address) => self.call_subroutine(address)?,
            Opcode::Return => self.r#return()?,
            Opcode::SkipInstructionIfEqual(r0, immediate) => {
                if self.registers[r0 as usize] == immediate {
                    self.pc += 2
                }
            }
            Opcode::SkipInstructionIfNotEqual(r0, immediate) => {
                if self.registers[r0 as usize] != immediate {
                    self.pc += 2
                }
            }
            Opcode::SkipInstructionIfRegistersEqual(r0, r1) => {
                if self.registers[r0 as usize] == self.registers[r1 as usize] {
                    self.pc += 2
                }
            }
            Opcode::SetRegister(r0, immediate) => {
                self.registers[r0 as usize] = immediate;
            }
            Opcode::AddToRegister(r0, immediate) => {
                let register = &mut self.registers[r0 as usize];
                *register = register.wrapping_add(immediate);
            }
            Opcode::CopyRegisters(r0, r1) => {
                self.registers[r0 as usize] = self.registers[r1 as usize]
            }
            Opcode::OrRegisters(r0, r1) => {
                self.registers[r0 as usize] |= self.registers[r1 as usize]
            }
            Opcode::AndRegisters(r0, r1) => {
                self.registers[r0 as usize] &= self.registers[r1 as usize]
            }
            Opcode::XorRegisters(r0, r1) => {
                self.registers[r0 as usize] ^= self.registers[r1 as usize]
            }
            Opcode::AddRegisters(r0, r1) => {
                let result =
                    self.registers[r0 as usize].overflowing_add(self.registers[r1 as usize]);
                (self.registers[r0 as usize], self.registers[15]) = (result.0, result.1 as u8);
            }
            Opcode::SubtractRegisters(r0, r1) => {
                let result =
                    self.registers[r0 as usize].overflowing_sub(self.registers[r1 as usize]);
                (self.registers[r0 as usize], self.registers[15]) = (result.0, !result.1 as u8);
            }
            Opcode::ShiftRegisterRight(r0, _r1) => {
                let r0 = &mut self.registers[r0 as usize];
                let bit = *r0 & 0x1;

                *r0 >>= 1;
                self.registers[15] = bit;
            }
            Opcode::SubtractRegistersReversed(r0, r1) => {
                let result =
                    self.registers[r1 as usize].overflowing_sub(self.registers[r0 as usize]);
                (self.registers[r0 as usize], self.registers[15]) = (result.0, !result.1 as u8);
            }
            Opcode::ShiftRegisterLeft(r0, _r1) => {
                let r0 = &mut self.registers[r0 as usize];
                let bit = *r0 & 0x80;

                *r0 <<= 1;
                self.registers[15] = (bit != 0) as u8;
            }
            Opcode::SkipInstructionIfRegistersNotEqual(r0, r1) => {
                if self.registers[r0 as usize] != self.registers[r1 as usize] {
                    self.pc += 2;
                }
            }
            Opcode::SetMemoryAddress(immediate) => {
                self.address = immediate;
            }
            Opcode::JumpToMemoryAddress(immediate) => {
                self.pc = (immediate + self.registers[0] as u16) as usize;
            }
            Opcode::SetRegisterRandom(r0, immediate) => {
                let number = random_range(0..=255);
                self.registers[r0 as usize] = (number & immediate as u32) as u8;
            }
            Opcode::DrawSprite(r0, r1, immediate) => {
                let (x, y, height) = (
                    self.registers[r0 as usize] as usize,
                    self.registers[r1 as usize] as usize,
                    immediate as usize,
                );

                self.registers[15] = 0;

                for dy in 0..height {
                    let sprite = self.memory[self.address as usize + dy];
                    for dx in 0..8 {
                        let sprite_bit = (sprite >> (7 - dx)) & 1;

                        if sprite_bit == 1 && self.display[y + dy][x + dx] == 1 {
                            self.registers[15] = 1;
                        }

                        self.display[y + dy][x + dx] ^= sprite_bit;
                    }
                }
            }
            Opcode::SkipInstructionIfKeyDown(r0) => {
                let input_address = self.registers[r0 as usize] & 15;
                let input = self.input[input_address as usize];

                if input != 0 {
                    self.pc += 2;
                }
            }
            Opcode::SkipInstructionIfKeyUp(r0) => {
                let input_address = self.registers[r0 as usize] & 15;
                let input = self.input[input_address as usize];

                if input == 0 {
                    self.pc += 2;
                }
            }
            Opcode::StoreDelayTimerToRegister(r0) => {
                self.registers[r0 as usize] = self.delay_timer;
            }
            Opcode::HaltAndStoreKeypressIntoRegister(_r0) => {
                self.awaiting_keypress = true;
                todo!();
            }
            Opcode::SetDelayTimerToRegister(r0) => {
                self.delay_timer = self.registers[r0 as usize];
            }
            Opcode::SetSoundTimerToRegister(r0) => {
                self.sound_timer = self.registers[r0 as usize];
            }
            Opcode::AddRegisterToMemoryAddress(r0) => {
                let result = self
                    .address
                    .overflowing_add(self.registers[r0 as usize] as u16);
                self.address = result.0;
            }
            Opcode::SetMemoryAddressToSpriteFromRegister(_) => {
                unimplemented!()
            }
            _ => Err(format!("Unknown instruction {:?}", opcode))?,
        };

        Ok(EmulatorStatus::Working)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_update_working {
        ($e: expr) => {
            assert_eq!($e.update(), Ok(EmulatorStatus::Working))
        };
    }

    macro_rules! assert_update_done {
        ($e: expr) => {
            assert_eq!($e.update(), Ok(EmulatorStatus::Done))
        };
    }

    struct Ts {
        rom_path: &'static str,
    }

    fn init() -> Ts {
        let rom_path = "./src/examples/chip8-roms/games/Pong (1 player).ch8";

        Ts { rom_path }
    }

    #[test]
    fn load_rom() {
        let Ts { rom_path, .. } = init();
        let mut emulator = Emulator::new();

        emulator.load_rom(rom_path).unwrap();
    }

    #[test]
    fn clear_screen() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::ClearScreen])
            .with_display([[1; 64]; 32]);

        assert_update_working!(emulator);
        assert_eq!(emulator.display, [[0; 64]; 32]);
    }

    #[test]
    fn opcode_goto() {
        let mut emulator = Emulator::new().with_opcodes(vec![Opcode::Goto(42)]);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 42);
    }

    #[test]
    fn opcode_skip_if_register_immediate() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 4 + 0x200)
    }

    #[test]
    fn opcode_skip_if_register_immediate_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 69);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2 + 0x200);
    }

    #[test]
    fn opcode_skip_if_register_not_immediate() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 4 + 0x200);
    }

    #[test]
    fn opcode_skip_if_register_not_immediate_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfNotEqual(0, 42)])
            .with_register_as(0, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2 + 0x200);
    }

    #[test]
    fn opcode_skip_if_registers_equal() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 4 + 0x200);
    }

    #[test]
    fn opcode_skip_if_registers_equal_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 69);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2 + 0x200);
    }

    #[test]
    fn opcode_set_register() {
        let mut emulator = Emulator::new().with_opcodes(vec![Opcode::SetRegister(0, 42)]);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
    }

    #[test]
    fn opcode_add_to_register() {
        let mut emulator = Emulator::new().with_opcodes(vec![
            Opcode::SetRegister(0, 254),
            Opcode::AddToRegister(0, 1),
            Opcode::AddToRegister(0, 1),
        ]);

        assert_eq!(emulator.registers[0], 0);
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 254);
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 255);
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0);
    }

    #[test]
    fn opcode_copy_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::CopyRegisters(0, 1)])
            .with_register_as(1, 42);

        assert_eq!(emulator.registers[0], 0);
        assert_eq!(emulator.registers[1], 42);
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[1], 42);
        assert_eq!(emulator.registers[1], 42)
    }

    #[test]
    fn opcode_or_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::OrRegisters(0, 1)])
            .with_register_as(0, 1)
            .with_register_as(1, 2);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 3);
    }

    #[test]
    fn opcode_and_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::AndRegisters(0, 1)])
            .with_register_as(0, 1)
            .with_register_as(1, 2);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0);
    }

    #[test]
    fn opcode_xor_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::AndRegisters(0, 1)])
            .with_register_as(0, 4)
            .with_register_as(1, 6);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 4);
    }

    #[test]
    fn opcode_add_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::AddRegisters(0, 1)])
            .with_register_as(0, 40)
            .with_register_as(1, 2);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 0);
    }

    #[test]
    fn opcode_add_registers_with_overflow() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::AddRegisters(0, 1)])
            .with_register_as(0, 255)
            .with_register_as(1, 43);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 1);
    }

    #[test]
    fn opcode_subtract_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegisters(0, 1)])
            .with_register_as(0, 255)
            .with_register_as(1, 213);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 1);
    }

    #[test]
    fn opcode_subtract_registers_with_underflow() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegisters(0, 1)])
            .with_register_as(0, 0)
            .with_register_as(1, 214);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 0);
    }

    #[test]
    fn opcode_shift_register_right() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::ShiftRegisterRight(0, 1)])
            .with_register_as(0, 85);

        assert_eq!(emulator.registers[0], 85);
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 1);
    }

    #[test]
    fn opcode_subtract_registers_reversed() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegistersReversed(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 84);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 1);
    }

    #[test]
    fn opcode_subtract_registers_reversed_with_underflow() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegistersReversed(0, 1)])
            .with_register_as(0, 214)
            .with_register_as(1, 0);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 0);
    }

    #[test]
    fn opcode_shift_register_left() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::ShiftRegisterLeft(0, 1)])
            .with_register_as(0, 0b10010101);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
        assert_eq!(emulator.registers[15], 1);
    }

    #[test]
    fn opcode_skip_if_registers_not_equal() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersNotEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 0);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 4 + 0x200);
    }

    #[test]
    fn opcode_skip_if_registers_not_equal_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersNotEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2 + 0x200);
    }

    #[test]
    fn opcode_set_memory_address() {
        let mut emulator = Emulator::new().with_opcodes(vec![Opcode::SetMemoryAddress(0xfef)]);

        assert_update_working!(emulator);
        assert_eq!(emulator.address, 0xfef);
    }

    #[test]
    fn opcode_jump_to_memory_address() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::JumpToMemoryAddress(0xfef)])
            .with_register_as(0, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 0xfef + 42);
    }

    #[test]
    fn opcode_set_register_random() {
        let mut values = Vec::new();

        for _ in 0..10000 {
            let mut emulator =
                Emulator::new().with_opcodes(vec![Opcode::SetRegisterRandom(0, 0xFF)]);

            assert_update_working!(emulator);
            values.push(emulator.registers[0]);
        }

        let mut average: u64 = 0u64;

        for v in &values {
            average += *v as u64;
        }

        average /= values.len() as u64;

        assert!(average >= 125 && average <= 130);
    }

    #[test]
    fn opcode_draw_sprite() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfKeyDown(0)])
            .with_register_as(0, 0x1F)
            .with_input_as(0xF, 1);

        assert_update_working!(emulator);
    }

    #[test]
    fn opcode_skip_if_key_down() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfKeyDown(0)])
            .with_register_as(0, 0x1F)
            .with_input_as(0xF, 1);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 4 + 0x200);
    }

    #[test]
    fn opcode_skip_if_key_down_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfKeyDown(0)])
            .with_register_as(0, 0x1F)
            .with_input_as(0xF, 0);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2 + 0x200);
    }
}
