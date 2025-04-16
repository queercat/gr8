use std::fs;

use super::opcode::Opcode;

#[derive(Debug)]
struct Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    address: u16,
    display: [[u8; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    input: [u8; 16],
    stack: [u8; 48],
    sp: usize,
    opcodes: Vec<Opcode>,
    pc: usize,
}

impl From<Vec<Opcode>> for Emulator {
    fn from(opcodes: Vec<Opcode>) -> Self {
        let mut emulator = Emulator::new();
        emulator.opcodes = opcodes;

        emulator
    }
}

#[derive(Debug, PartialEq)]
enum EmulatorStatus {
    Working,
    Done,
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
            sp: 0,
            opcodes: Vec::new(),
            pc: 0,
        }
    }

    fn with_opcodes(mut self, opcodes: Vec<Opcode>) -> Self {
        self.opcodes = opcodes;
        self
    }

    fn with_register_as(mut self, r: u8, v: u8) -> Self {
        self.registers[r as usize] = v;
        self
    }

    fn with_display(mut self, display: [[u8; 64]; 32]) -> Self {
        self.display = display;
        self
    }

    pub fn load_rom(&mut self, path_to_rom: &str) -> Result<(), String> {
        let rom_data = fs::read(path_to_rom).map_err(|e| e.to_string())?;
        self.opcodes = Opcode::decode(&rom_data)?;

        Ok(())
    }

    fn clear_screen(&mut self) {
        self.display = [[0; 64]; 32]
    }

    fn goto(&mut self, address: u16) {
        self.pc = address as usize;
    }

    fn call_subroutine(&mut self, address: u16) -> Result<(), String> {
        if self.sp >= self.stack.len() {
            return Err("Stack overflow!".to_string());
        }

        self.stack[self.sp] = self.pc as u8;
        self.sp += 1;

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

    fn update(&mut self) -> Result<EmulatorStatus, String> {
        if self.pc >= self.opcodes.len() {
            return Ok(EmulatorStatus::Done);
        }

        let instruction = &self.opcodes[self.pc];

        self.pc += 1;

        match *instruction {
            Opcode::ClearScreen => self.clear_screen(),
            Opcode::Goto(address) => self.goto(address),
            Opcode::CallSubroutine(address) => self.call_subroutine(address)?,
            Opcode::Return => self.r#return()?,
            Opcode::SkipInstructionIfEqual(r0, immediate) => {
                if self.registers[r0 as usize] == immediate {
                    self.pc += 1
                }
            }
            Opcode::SkipInstructionIfNotEqual(r0, immediate) => {
                if self.registers[r0 as usize] != immediate {
                    self.pc += 1
                }
            }
            Opcode::SkipInstructionIfRegistersEqual(r0, r1) => {
                if self.registers[r0 as usize] == self.registers[r1 as usize] {
                    self.pc += 1
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
            _ => Err(format!("Unknown instruction {:?}", instruction))?,
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
        assert_eq!(emulator.pc, 2);
    }

    #[test]
    fn opcode_skip_if_register_immediate_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 69);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 1);
    }

    #[test]
    fn opcode_skip_if_register_not_immediate() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2);
    }

    #[test]
    fn opcode_skip_if_register_not_immediate_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfEqual(0, 42)])
            .with_register_as(0, 69);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 1);
    }

    #[test]
    fn opcode_skip_if_registers_equal() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 42);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 2);
    }

    #[test]
    fn opcode_skip_if_registers_equal_negative() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SkipInstructionIfRegistersEqual(0, 1)])
            .with_register_as(0, 42)
            .with_register_as(1, 69);

        assert_update_working!(emulator);
        assert_eq!(emulator.pc, 1);
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

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
    }

    #[test]
    fn opcode_or_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::OrRegisters(0, 1)])
            .with_register_as(0, 0b1010)
            .with_register_as(1, 0b0101);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0b1111);
    }

    #[test]
    fn opcode_and_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::AndRegisters(0, 1)])
            .with_register_as(0, 0b1010)
            .with_register_as(1, 0b1100);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0b1000);
    }

    #[test]
    fn opcode_xor_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::XorRegisters(0, 1)])
            .with_register_as(0, 0b1100)
            .with_register_as(1, 0b1010);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0b0110);
    }

    #[test]
    fn opcode_subtract_registers() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegisters(0, 1)])
            .with_register_as(0, 10)
            .with_register_as(1, 5);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 5);
        assert_eq!(emulator.registers[15], 1); // No borrow, VF = 1

        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::SubtractRegisters(0, 1)])
            .with_register_as(0, 5)
            .with_register_as(1, 10);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 251); // 5 - 10 = -5 = 251 in two's complement
        assert_eq!(emulator.registers[15], 0); // Borrow occurred, VF = 0
    }

    #[test]
    fn opcode_shift_register_right() {
        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::ShiftRegisterRight(0, 1)])
            .with_register_as(0, 0b101);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0b10); // 5 >> 1 = 2
        assert_eq!(emulator.registers[15], 1); // LSB was 1

        let mut emulator = Emulator::new()
            .with_opcodes(vec![Opcode::ShiftRegisterRight(0, 1)])
            .with_register_as(0, 0b110);

        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 0b11); // 6 >> 1 = 3
        assert_eq!(emulator.registers[15], 0); // LSB was 0
    }
}
