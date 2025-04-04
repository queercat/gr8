#[derive(Debug, PartialEq)]
pub enum Opcode {
    /// 0NNN: Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.
    CallMachineCodeRoutine(u16),
    /// 00E0: Clears the screen.
    ClearScreen,
    /// 00EE: Returns from a subroutine.
    Return,
    /// 1NNN: Jumps to address NNN.
    Goto(u16),
    /// 2NNN: Calls subroutine at NNN.
    CallSubroutine(u16),
    /// 3XNN: Skips the next instruction if VX equals NN (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfEqual(u8, u8),
    /// 4XNN: Skips the next instruction if VX does not equal NN (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfNotEqual(u8, u8),
    /// 5XY0: Skips the next instruction if VX equals VY (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfRegistersEqual(u8, u8),
    /// 6XNN: Sets VX to NN.
    SetRegister(u8, u8),
    /// 7XNN: Adds NN to VX (carry flag is not changed).
    AddToRegister(u8, u8),
    /// 8XY0: Sets VX to the value of VY.
    CopyRegisters(u8, u8),
    /// 8XY1: Sets VX to VX or VY. (bitwise OR operation).
    OrRegisters(u8, u8),
    /// 8XY2: Sets VX to VX and VY. (bitwise AND operation).
    AndRegisters(u8, u8),
    /// 8XY3: Sets VX to VX xor VY.
    XorRegisters(u8, u8),
    /// 8XY4: Adds VY to VX. VF is set to 1 when there's an overflow, and to 0 when there is not.
    AddRegisters(u8, u8),
    /// 8XY5: VY is subtracted from VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VX >= VY and 0 if not).
    SubtractRegisters(u8, u8),
    /// 8XY6: Shifts VX to the right by 1, then stores the least significant bit of VX prior to the shift into VF.
    ShiftRegisterRight(u8, u8),
    /// 8XY7: Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VY >= VX).
    SubtractRegistersReversed(u8, u8),
    /// 8XYE: Shifts VX to the left by 1, then sets VF to 1 if the most significant bit of VX prior to that shift was set, or to 0 if it was unset.
    ShiftRegisterLeft(u8, u8),
    /// 9XY0: Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block).
    SkipInstructionIfRegistersNotEqual(u8, u8),
    /// ANNN: Sets I to the address NNN.
    SetMemoryAddress(u16),
    /// BNNN: Jumps to the address NNN plus V0.
    JumpToMemoryAddress(u16),
    /// CXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    SetRegisterRandom(u8, u8),
    /// DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    /// Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction.
    /// As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen.
    DrawSprite(u8, u8, u8),
    /// EX9E: Skips the next instruction if the key stored in VX(only consider the lowest nibble) is pressed (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfKeyDown(u8),
    /// EXA1: Skips the next instruction if the key stored in VX(only consider the lowest nibble) is not pressed (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfKeyUp(u8),
    /// FX07: Sets VX to the value of the delay timer.
    StoreDelayTimerToRegister(u8),
    /// FX0A: A key press is awaited, and then stored in VX (blocking operation, all instruction halted until next key event, delay and sound timers should continue processing).
    HaltAndStoreKeypressIntoRegister(u8),
    /// FX15: Sets the delay timer to VX.
    SetDelayTimerToRegister(u8),
    /// FX18: Sets the sound timer to VX.
    SetSoundTimerToRegister(u8),
    /// FX1E: Adds VX to I. VF is not affected.
    AddRegisterToMemoryAddress(u8),
    /// FX29: Sets I to the location of the sprite for the character in VX(only consider the lowest nibble). Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    SetMemoryAddressToSpriteFromRegister(u8),
    /// FX33: Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.[
    SetMemoryAddressToBinaryEncodedDecimalFromRegister(u8),
    /// FX55: Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    DumpRegistersIntoMemoryUpToRegister(u8),
    /// FX65: Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified.
    DumpMemoryIntoRegistersUpToRegister(u8),
}

impl Opcode {
    pub fn decode(data: &Vec<u8>) -> Result<Vec<Opcode>, &'static str> {
        let mut index = 0usize;
        let mut instructions = Vec::<Opcode>::new();

        while index < data.len() {
            if index + 1 >= data.len() {
                return Err("Malformed ROM, expected valid byte but instead found half-byte.");
            }

            let byte = (data[index], data[index + 1]);
            let bits = (
                (byte.0 & 0xF0) >> 4,
                byte.0 & 0x0F,
                (byte.1 & 0xF0) >> 4,
                byte.1 & 0x0F,
            );

            match Opcode::decode_bits(bits) {
                Ok(opcode) => instructions.push(opcode),
                Err(e) => return Err(e),
            };

            index += 2;
        }

        Ok(instructions)
    }

    fn decode_triple_hex_bit(n0: u8, n1: u8, n2: u8) -> u16 {
        ((n0 as u16) << 8) +
        ((n1 as u16) << 4) +
        (n2 as u16)
    }

    fn decode_bits(bits: (u8, u8, u8, u8)) -> Result<Opcode, &'static str> {
        let opcode = match bits {
            (0x0, 0x0, 0xE, 0x0) => Opcode::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => Opcode::Return,
            (0x0, n0, n1, n2) => Opcode::CallMachineCodeRoutine(Opcode::decode_triple_hex_bit(n0, n1, n2)),
            (0x1, n0, n1, n2) => Opcode::Goto(Opcode::decode_triple_hex_bit(n0, n1, n2)),
            (0x2, n0, n1, n2) => Opcode::CallSubroutine(Opcode::decode_triple_hex_bit(n0, n1, n2)),
            (0x3, r0, n0, n1) => Opcode::SkipInstructionIfEqual(r0, (n0 << 4) + n1),
            (0x4, r0, n0, n1) => Opcode::SkipInstructionIfNotEqual(r0, (n0 << 4) + n1),
            (0x5, r0, r1, 0) => Opcode::SkipInstructionIfRegistersEqual(r0, r1),
            (0x6, r0, n0, n1) => Opcode::SetRegister(r0, (n0 << 4) + n1),
            (0x7, r0, n0, n1) => Opcode::AddToRegister(r0, (n0 << 4) + n1),
            (0x8, r0, r1, 0) => Opcode::CopyRegisters(r0, r1),
            (0x8, r0, r1, 1) => Opcode::OrRegisters(r0, r1),
            (0x8, r0, r1, 2) => Opcode::AndRegisters(r0, r1),
            (0x8, r0, r1, 3) => Opcode::XorRegisters(r0, r1),
            (0x8, r0, r1, 4) => Opcode::AddRegisters(r0, r1),
            (0x8, r0, r1, 5) => Opcode::SubtractRegisters(r0, r1),
            (0x8, r0, r1, 6) => Opcode::ShiftRegisterRight(r0, r1),
            (0x8, r0, r1, 7) => Opcode::SubtractRegistersReversed(r0, r1),
            (0x8, r0, r1, 0xE) => Opcode::ShiftRegisterLeft(r0, r1),
            (0x9, r0, r1, 0x0) => Opcode::SkipInstructionIfRegistersNotEqual(r0, r1),
            (0xA, n0, n1, n2) => Opcode::SetMemoryAddress(Opcode::decode_triple_hex_bit(n0, n1, n2)),
            (0xB, n0, n1, n2) => Opcode::JumpToMemoryAddress(Opcode::decode_triple_hex_bit(n0, n1, n2)),
            (0xC, r0, n0, n1) => Opcode::SetRegisterRandom(r0, (n0 << 4) + n1),
            (0xD, r0, r1, n0) => Opcode::DrawSprite(r0, r1, n0),
            (0xE, r0, 0x9, 0xE) => Opcode::SkipInstructionIfKeyDown(r0),
            (0xE, r0, 0xA, 0x1) => Opcode::SkipInstructionIfKeyUp(r0),
            (0xF, r0, 0x0, 0x7) => Opcode::StoreDelayTimerToRegister(r0),
            (0xF, r0, 0x0, 0xA) => Opcode::HaltAndStoreKeypressIntoRegister(r0),
            (0xF, r0, 0x1, 0x5) => Opcode::SetDelayTimerToRegister(r0),
            (0xF, r0, 0x1, 0x8) => Opcode::SetSoundTimerToRegister(r0),
            (0xF, r0, 0x1, 0xE) => Opcode::AddRegisterToMemoryAddress(r0),
            (0xF, r0, 0x2, 0x9) => Opcode::SetMemoryAddressToSpriteFromRegister(r0),
            (0xF, r0, 0x3, 0x3) => Opcode::SetMemoryAddressToBinaryEncodedDecimalFromRegister(r0),
            (0xF, r0, 0x5, 0x5) => Opcode::DumpRegistersIntoMemoryUpToRegister(r0),
            (0xF, r0, 0x6, 0x5) => Opcode::DumpMemoryIntoRegistersUpToRegister(r0),

            _ => return Err("Unsupported instruction!")
        };

        Ok(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_decode_clear_display_instruction() {
        assert_eq!(
            Opcode::decode(&vec![0x00 as u8, 0xE0 as u8]),
            Ok(vec![Opcode::ClearScreen])
        )
    }
}
