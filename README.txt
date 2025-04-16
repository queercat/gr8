# GR8: CHIP-8 Emulator

## Overview
GR8 is a CHIP-8 emulator implemented in Rust using the Macroquad game engine. CHIP-8 is a simple, interpreted programming language that was first designed in the 1970s for early microcomputers. This emulator allows you to run classic CHIP-8 games and applications in a modern environment.

## Features
- Complete implementation of the CHIP-8 instruction set
- Graphics rendering using Macroquad
- ROM loading functionality
- Support for all standard CHIP-8 operations

## Project Structure
- `src/main.rs` - Application entry point and main loop
- `src/emulator/` - Core emulator implementation
  - `emulator.rs` - Contains the Emulator struct that implements the CHIP-8 VM
  - `opcode.rs` - Defines the Opcode enum and ROM decoding functionality
- `src/examples/chip8-roms/` - Git submodule containing example CHIP-8 ROMs

## Requirements
- Rust (latest stable version recommended)
- Macroquad dependencies (see https://github.com/not-fl3/macroquad)

## Building and Running
1. Clone the repository:
   ```
   git clone https://github.com/queercat/gr8.git
   cd gr8
   git submodule update --init --recursive
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the tests:
   ```
   cargo test
   ```

## Running with ROMs
The emulator can load ROMs from the file system. Example usage in tests:
```rust
let mut emulator = Emulator::new();
emulator.load_rom("./src/examples/chip8-roms/games/Pong (1 player).ch8").unwrap();
```

## About CHIP-8
CHIP-8 is an interpreted programming language developed in the 1970s for programming games. It was designed to make game development easier on early microcomputers. A CHIP-8 virtual machine has:

- 4KB of memory
- 16 8-bit registers (V0-VF)
- A 64x32 pixel monochrome display
- A 16-key hexadecimal keypad
- Two timers (delay and sound)
- A stack for subroutine calls

More information: https://en.wikipedia.org/wiki/CHIP-8

## License
[Add license information if available]
