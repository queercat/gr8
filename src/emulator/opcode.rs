enum Opcode {
    /// 0NNN: Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.
    CallMachineCodeRoutine(i16),
    /// 00E0: Clears the screen.
    ClearScreen,
    /// 00EE: Returns from a subroutine.
    Return,
    /// 1NNN: Jumps to address NNN.
    Goto(i16),
    /// 2NNN: Calls subroutine at NNN.
    CallSubroutine(i16),
    /// 3XNN: Skips the next instruction if VX equals NN (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfEqual(i8, i8),
    /// 4XNN: Skips the next instruction if VX does not equal NN (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfNotEqual(i8, i8),
    /// 5XY0: Skips the next instruction if VX equals VY (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfRegistersEqual(i8, i8),
    /// 6XNN: Sets VX to NN.
    SetRegister(i8, i8),
    /// 7XNN: Adds NN to VX (carry flag is not changed).
    AddToRegister(i8, i8),
    /// 8XY0: Sets VX to the value of VY.
    CopyRegisters(i8, i8),
    /// 8XY1: Sets VX to VX or VY. (bitwise OR operation).
    OrRegisters(i8, i8),
    /// 8XY2: Sets VX to VX and VY. (bitwise AND operation).
    AndRegisters(i8, i8),
    /// 8XY3: Sets VX to VX xor VY.
    XorRegisters(i8, i8),
    /// 8XY4: Adds VY to VX. VF is set to 1 when there's an overflow, and to 0 when there is not.
    AddRegisters(i8, i8),
    /// 8XY5: VY is subtracted from VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VX >= VY and 0 if not).
    SubtractRegisters(i8, i8),
    /// 8XY6: Shifts VX to the right by 1, then stores the least significant bit of VX prior to the shift into VF.
    ShiftRegisterRight(i8, i8),
    /// 8XY7: Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VY >= VX).
    SubtractRegistersReversed(i8, i8),
    /// 8XYE: Shifts VX to the left by 1, then sets VF to 1 if the most significant bit of VX prior to that shift was set, or to 0 if it was unset.
    ShiftRegisterLeft(i8, i8),
    /// 9XY0: Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block).
    SkipInstructionIfRegistersNotEqual(i8, i8),
    /// ANNN: Sets I to the address NNN.
    SetMemoryAddress(i16),
    /// BNNN: Jumps to the address NNN plus V0.
    JumpToMemoryAddress(i16),
    /// CXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    SetRegisterRandom(i8, i8),
    /// DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. 
    /// Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. 
    /// As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen.
    DrawSprite(i8, i8, i8),
    /// EX9E: Skips the next instruction if the key stored in VX(only consider the lowest nibble) is pressed (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfKeyDown(i8),
    /// EXA1: Skips the next instruction if the key stored in VX(only consider the lowest nibble) is not pressed (usually the next instruction is a jump to skip a code block).
    SkipInstructionIfKeyUp(i8),
    /// FX07: Sets VX to the value of the delay timer.
    StoreDelayTimerToRegister(i8),
    /// FX0A: A key press is awaited, and then stored in VX (blocking operation, all instruction halted until next key event, delay and sound timers should continue processing).
    HaltAndStoreKeypressIntoRegister(i8),
    /// FX15: Sets the delay timer to VX.
    SetDelayTimerToRegister(i8),
    /// FX18: Sets the sound timer to VX.
    SetSoundTimerToRegister(i8),
    /// FX1E: Adds VX to I. VF is not affected.
    AddRegisterToMemoryAddress(i8),
    /// FX29: Sets I to the location of the sprite for the character in VX(only consider the lowest nibble). Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    SetMemoryAddressToSpriteFromRegister(i8),
    /// FX33: Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.[
    SetMemoryAddressToBinaryEncodedDecimalFromRegister(i8),
    /// FX55: Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    DumpRegistersIntoMemoryUpToRegister(i8),
    /// FX65: Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified.
    DumpMemoryIntoRegistersUpToRegister(i8),
}