# GR8 Emulator Bug Fix Reference

This document outlines identified bugs and issues in the GR8 Chip-8/Super Chip-8 emulator implementation that need to be addressed.

## Critical Issues

1. **Incomplete Keypress Handling**
   - **File**: `emulator.rs`
   - **Issue**: The `HaltAndStoreKeypressIntoRegister` opcode implementation only sets `awaiting_keypress` to false but doesn't store any keypress value in the register.
   - **Fix**: Implement proper keypress detection and storage in the specified register.
   ```rust
   Opcode::HaltAndStoreKeypressIntoRegister(r0) => {
       self.awaiting_keypress = false;
       // Add code to store the detected keypress in register r0
   }
   ```

2. **Sound Timer Encoding Error**
   - **File**: `opcode.rs`
   - **Issue**: Incorrect encoding for the `SetSoundTimerToRegister` opcode (using `0xF010` instead of `0xF018`).
   - **Fix**: Correct the encoding:
   ```rust
   Opcode::SetSoundTimerToRegister(data) => (0xF018 | (data as u16) << 8).to_bits(),
   ```

3. **DrawSprite Opcode Encoding Issue**
   - **File**: `opcode.rs`
   - **Issue**: Potential issue with how the bits are combined in the DrawSprite opcode.
   - **Fix**: Update the encoding to properly handle registers and height:
   ```rust
   Opcode::DrawSprite(l, m, r) => (0xD0 | l, (m << 4) | r),
   ```

## Functional Issues

4. **Missing Input Handling**
   - **File**: `main.rs`
   - **Issue**: No code to handle keyboard input and update the `emulator.input` array.
   - **Fix**: Add keyboard input handling to map keys to the Chip-8 keypad and update the input array.

5. **Timer Initialization Issue**
   - **File**: `emulator.rs`
   - **Issue**: Timers and their last updated times are both initialized to 0, which could cause issues with the first timer update.
   - **Fix**: Initialize the last updated times to the current time when creating a new emulator.

6. **Potential Overflow in Display Coordinates**
   - **File**: `emulator.rs`
   - **Issue**: Inconsistent handling of out-of-bounds coordinates in the `DrawSprite` implementation.
   - **Fix**: Use consistent boundary checking for both x and y coordinates:
   ```rust
   if y + dy >= DISPLAY_HEIGHT || x + dx >= DISPLAY_WIDTH {
       continue;
   }
   ```

## Testing Issues

7. **Incorrect XOR Test**
   - **File**: `emulator.rs` (tests section)
   - **Issue**: The test for `opcode_xor_registers` is actually testing `AndRegisters` instead of `XorRegisters`.
   - **Fix**: Update the test to use the correct opcode:
   ```rust
   #[test]
   fn opcode_xor_registers() {
       let mut emulator = Emulator::new()
           .with_opcodes(vec![Opcode::XorRegisters(0, 1)])
           .with_register_as(0, 4)
           .with_register_as(1, 6);

       assert_update_working!(emulator);
       assert_eq!(emulator.registers[0], 2); // 4 XOR 6 = 2
   }
   ```

## Design Issues

8. **Inconsistent Color Handling**
   - **File**: `main.rs`
   - **Issue**: The color match statement always returns WHITE regardless of the x value.
   - **Fix**: Either remove the unnecessary match statement or implement proper color variation.

9. **Potential Issue with SystemTime**
   - **File**: `main.rs`
   - **Issue**: Using a single SystemTime instance for the entire program duration could lead to overflow.
   - **Fix**: Consider resetting the time periodically or using a different timing approach.

10. **Missing Super Chip-8 Features**
    - **Issue**: Despite being described as a Super Chip-8 emulator, extended features are not implemented.
    - **Fix**: Implement Super Chip-8 specific instructions like scrolling, extended resolution (128x64), etc.

11. **Unimplemented CallMachineCodeRoutine**
    - **File**: `emulator.rs`
    - **Issue**: The `CallMachineCodeRoutine` opcode is marked as unimplemented with a panic message.
    - **Fix**: Either implement a proper no-op for this instruction or add better error handling.

12. **Potential Issue with Shift Instructions**
    - **File**: `emulator.rs`
    - **Issue**: Shift instructions (8XY6 and 8XYE) shift VX directly instead of shifting VY and storing in VX.
    - **Fix**: Consider adding a configuration option to support both modern and original Chip-8 behavior.

## Next Steps

1. Address the critical issues first (1-3)
2. Fix the functional issues (4-6)
3. Correct the test cases (7)
4. Consider the design improvements (8-12)

After fixing these issues, your GR8 emulator should have improved compatibility with Chip-8 ROMs and better overall reliability.
