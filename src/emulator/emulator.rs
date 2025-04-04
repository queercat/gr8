#[derive(Debug)]
struct Emulator {
    memory: [i8; 4096],
    registers: [i8; 16],
    address: i16,
    display: [[i8; 64]; 32],
    delay_timer: i8,
    sound_timer: i8,
    input: [i8; 16],
    stack: [i8; 48]
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
            stack: [0; 48]
        }
    }

    /// The core update loop for the emulator.
    fn update(&mut self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct_emulator() {
        let _emulator = Emulator::new();
    }
}