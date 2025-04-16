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
    opcodes: Option<Vec<Opcode>>,  // Make optional for backward compatibility
    pc: u16,                       // Change to u16 to match memory addressing
    program_start: u16,            // Store the start address of the program in memory
}

impl From<Vec<Opcode>> for Emulator {
    fn from(opcodes: Vec<Opcode>) -> Self {
        let mut emulator = Emulator::new();
        emulator.opcodes = Some(opcodes);
        
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
            opcodes: None,
            pc: 0,                   // Start at 0 for tests, will be set to 0x200 when loading ROM
            program_start: 0x200,    // Standard program start address
        }
    }

    fn with_opcodes(mut self, opcodes: Vec<Opcode>) -> Self {
        self.opcodes = Some(opcodes);
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
    
    fn read_memory_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn read_memory_word(&self, address: u16) -> u16 {
        let high = self.read_memory_byte(address) as u16;
        let low = self.read_memory_byte(address + 1) as u16;
        (high << 8) | low
    }

    fn write_memory_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    fn write_memory_word(&mut self, address: u16, value: u16) {
        self.write_memory_byte(address, ((value & 0xFF00) >> 8) as u8);
        self.write_memory_byte(address + 1, (value & 0x00FF) as u8);
    }

    pub fn load_rom(&mut self, path_to_rom: &str) -> Result<(), String> {
        let rom_data = fs::read(path_to_rom).map_err(|e| e.to_string())?;
        
        self.opcodes = Some(Opcode::decode(&rom_data)?);
        
        for (i, &byte) in rom_data.iter().enumerate() {
            if (self.program_start as usize + i) < self.memory.len() {
                self.memory[self.program_start as usize + i] = byte;
            } else {
                return Err("ROM too large for memory".to_string());
            }
        }
        
        self.pc = self.program_start;
        
        Ok(())
    }

    fn fetch_opcode(&self) -> Result<Opcode, String> {
        if (self.pc as usize) + 1 >= self.memory.len() {
            return Err("Attempted to read beyond memory bounds".to_string());
        }
        
        let byte1 = self.read_memory_byte(self.pc);
        let byte2 = self.read_memory_byte(self.pc + 1);
        
        Opcode::decode_opcode(byte1, byte2)
    }


    fn clear_screen(&mut self) {
        self.display = [[0; 64]; 32];
    }

    fn goto(&mut self, address: u16) {
        self.pc = address;
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

        self.goto(self.stack[self.sp - 1] as u16);
        self.sp -= 1;

        Ok(())
    }
    fn fetch_decode_execute(&mut self) -> Result<EmulatorStatus, String> {
        if (self.pc as usize) + 1 >= self.memory.len() {
            return Ok(EmulatorStatus::Done);
        }
        
        let opcode = self.fetch_opcode()?;
        
        self.pc += 2;
        
        match opcode {
            Opcode::ClearScreen => self.clear_screen(),
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
            _ => return Err(format!("Unsupported opcode: {:?}", opcode)),
        };
        
        Ok(EmulatorStatus::Working)
    }



    fn update(&mut self) -> Result<EmulatorStatus, String> {
        if let Some(opcodes) = &self.opcodes {
            if self.pc as usize >= opcodes.len() {
                return Ok(EmulatorStatus::Working);
            }

            let instruction = &opcodes[self.pc as usize];
            self.pc += 1;  // Increment PC

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
                _ => return Err(format!("Unknown instruction {:?}", instruction)),
            };

            Ok(EmulatorStatus::Working)
        } else {
            self.fetch_decode_execute()
        }
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
        let mut emulator = Emulator::new().with_opcodes(vec![
            Opcode::CopyRegisters(0, 1)
        ]);
        emulator.registers[1] = 42;
        
        assert_update_working!(emulator);
        assert_eq!(emulator.registers[0], 42);
    }
    
    #[test]
    fn fetch_decode_execute_cycle() {
        let mut emulator = Emulator::new();
        
        emulator.pc = 0x200;
        
        emulator.write_memory_byte(0x200, 0x00);
        emulator.write_memory_byte(0x201, 0xE0);
        
        emulator.opcodes = None;
        
        assert!(matches!(emulator.update(), Ok(EmulatorStatus::Working)));
        
        assert_eq!(emulator.pc, 0x202);
    }
}
