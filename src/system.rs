use ops::{get_op_group, OP_GROUPS};

use std::io::Write;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const COLOR_WIDTH: usize = 4;
const MAX_INDEX: usize = WIDTH * HEIGHT * COLOR_WIDTH;

pub struct System {
    v: [u8; 16],
    i: u16,
    pc: usize,
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    mem: [u8; 4096],
    vmem: [u8; MAX_INDEX],
    input: u16,
    previous_input: u16,
}

impl System {
    pub fn new() -> Self {
        let mut system = Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            mem: [0; 4096],
            vmem: [0; MAX_INDEX],
            input: 0,
            previous_input: 0,
        };

        // Write reserved interpreter memory
        (&mut system.mem[0x000..0x1FF])
            .write(&[
                0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                0x20, 0x60, 0x20, 0x20, 0x70, // 1
                0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                0xF0, 0x80, 0xF0, 0x80, 0x80, // F
            ])
            .unwrap();

        system
    }

    pub fn write_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.mem[0x200 + i] = rom[i];
        }
    }

    pub fn set_key(&mut self, key: u8, value: bool) {
        if value {
            self.input = self.input | (0x1 << key);
        } else {
            self.input = self.input & (0xFF ^ (0x1 << key));
        }
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        &self.vmem[..]
    }

    pub fn run_to_next_frame(&mut self) -> bool {
        let mut end_execution = false;

        for _ in 0..9 {
            end_execution |= self.tick();
        }

        end_execution
    }

    pub fn tick(&mut self) -> bool {
        let op = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;
        //println!("PC: {:04X} - op: {:04X}", self.pc, op);

        self.pc += 2;
        self.execute_op(op);

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        self.previous_input = self.input;

        (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16 != 0
    }

    fn execute_op(&mut self, op: u16) {
        let group = get_op_group(op);

        OP_GROUPS[group as usize](self, op & 0x0FFF);
    }
}

mod ops {
    use super::{System, COLOR_WIDTH, MAX_INDEX, WIDTH};
    use rand::Rng;
    use std::io::Write;

    /// All avaialable opcodes where the most-significant word (`0x0XXX` - `0xFXXX`) is the index. CHIP-8
    /// uses 16 bits for op codes. The most significant word (4-bits) is generally used to define the operation
    /// performed while the next 3 bytes are the "parameters".
    ///
    /// Each group (item within this list) handles all opcodes within the range of that most significant bit. For
    /// example, the `0x0` group will handle any opcodes ranging from `0x0000` to `0x0FFF`, if they exist. In the
    /// case of CHIP-8, the only two opcodes in the `0x0` group are `0x0E0` and `0x0EE`.
    pub const OP_GROUPS: [fn(&mut System, u16); 16] = [
        |system, op| {
            // 0x0XXX
            match split_op(op).1 {
                0xE0 => {
                    (&mut system.vmem[..]).write(&[0; MAX_INDEX]).unwrap();
                }
                0xEE => {
                    // Return from subroutine
                    system.pc = system.stack[system.sp];
                    system.sp = system.sp - 1;
                }
                _ => {}
            }
        },
        |system, op| {
            // 0x1XXX
            system.pc = (op & 0x0FFF) as usize;
        },
        |system, op| {
            // 0x2XXX
            // Put the program counter on the stack and jump
            system.sp = system.sp + 1;
            system.stack[system.sp] = system.pc;
            system.pc = (op & 0x0FFF) as usize;
        },
        |system, op| {
            // 0x3XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let value = combine_words(words[2], words[3]);

            if system.v[register] == value {
                system.pc += 2;
            }
        },
        |system, op| {
            // 0x4XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let value = combine_words(words[2], words[3]);

            if system.v[register] != value {
                system.pc += 2;
            }
        },
        |system, op| {
            // 0x5XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let cmp_register = words[2] as usize;

            if system.v[register] == system.v[cmp_register] {
                system.pc += 2;
            }
        },
        |system, op| {
            // 0x6XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let value = combine_words(words[2], words[3]);

            system.v[register] = value;
        },
        |system, op| {
            // 0x7XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let value = combine_words(words[2], words[3]);

            let (value, _) = system.v[register].overflowing_add(value);
            system.v[register] = value;
        },
        |system, op| {
            // 0x8XXX
            let words = get_op_words(op);
            let register1 = words[1] as usize;
            let register2 = words[2] as usize;

            match words[3] {
                0x0 => system.v[register1] = system.v[register2],
                0x1 => {
                    system.v[register1] = system.v[register1] | system.v[register2];
                }
                0x2 => {
                    system.v[register1] = system.v[register1] & system.v[register2];
                }
                0x3 => {
                    system.v[register1] = system.v[register1] ^ system.v[register2];
                }
                0x4 => {
                    let (value, overflow) =
                        system.v[register1].overflowing_add(system.v[register2]);

                    system.v[0xF] = if overflow { 1 } else { 0 };
                    system.v[register1] = value;
                }
                0x5 => {
                    let (value, overflow) =
                        system.v[register1].overflowing_sub(system.v[register2]);

                    system.v[0xF] = if overflow { 1 } else { 0 };
                    system.v[register1] = value;
                }
                0x6 => {
                    let (value, overflow) = system.v[register1].overflowing_shr(1);

                    system.v[0xF] = if overflow { 1 } else { 0 };
                    system.v[register1] = value;
                }
                0x7 => {
                    let (value, overflow) =
                        system.v[register2].overflowing_sub(system.v[register1]);

                    system.v[0xF] = if overflow { 1 } else { 0 };
                    system.v[register1] = value;
                }
                0xE => {
                    let (value, overflow) = system.v[register1].overflowing_shl(1);

                    system.v[0xF] = if overflow { 1 } else { 0 };
                    system.v[register1] = value;
                }
                _ => {}
            }
        },
        |system, op| {
            // 0x9XXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let cmp_register = words[2] as usize;

            if system.v[register] != system.v[cmp_register] {
                system.pc = system.pc + 2;
            }
        },
        |system, op| {
            // 0xAXXX
            system.i = (op & 0x0FFF) as u16;
        },
        |system, op| {
            // 0xBXXX
            system.pc = (system.v[0x0] + (op & 0x0FFF) as u8) as usize;
        },
        |system, op| {
            // 0xCXXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let value = combine_words(words[2], words[3]);

            let mut rng = rand::thread_rng();
            let rand_val = rng.gen_range(0, 255) as u8;

            system.v[register] = rand_val & value;
        },
        |system, op| {
            // 0xDXXX
            let words = get_op_words(op);
            let x = system.v[words[1] as usize] as usize;
            let y = system.v[words[2] as usize] as usize;
            let num_bytes = words[3] as usize;

            let bytes = &system.mem[system.i as usize..system.i as usize + num_bytes];

            let mut has_collision = false;

            for idx in 0..num_bytes {
                for split_byte in 0..8 {
                    if bytes[idx] & (0b1000_0000 >> split_byte) != 0 {
                        let vmem_idx =
                            ((((y + idx) * WIDTH) + (x + split_byte)) * COLOR_WIDTH) % MAX_INDEX;

                        if !has_collision {
                            has_collision = system.vmem[vmem_idx] > 0;
                        }
                        system.vmem[vmem_idx] ^= 0xFF;
                        system.vmem[vmem_idx + 1] ^= 0xFF;
                        system.vmem[vmem_idx + 2] ^= 0xFF;
                        system.vmem[vmem_idx + 3] ^= 0xFF;
                    }
                }
            }

            system.v[0xF] = if has_collision { 1 } else { 0 };
        },
        |system, op| {
            // 0xEXXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let instruction = combine_words(words[2], words[3]);

            match instruction {
                0x9E => {
                    if 2u16.pow(system.v[register] as u32) & system.input > 0 {
                        system.pc += 2;
                    }
                }
                0xA1 => {
                    if 2u16.pow(system.v[register] as u32) & system.input == 0 {
                        system.pc += 2;
                    }
                }
                _ => {}
            }
        },
        |system, op| {
            // 0xFXXX
            let words = get_op_words(op);
            let register = words[1] as usize;
            let instruction = combine_words(words[2], words[3]);

            match instruction {
                0x07 => system.v[register] = system.delay_timer,
                0x0A => {
                    if system.previous_input == system.input {
                        system.pc -= 2;
                    } else {
                        let key_diff = system.input ^ system.previous_input;
                        let key_value = (key_diff as f32).log2() as u8;

                        system.v[register] = key_value;
                    }
                }
                0x15 => system.delay_timer = system.v[register],
                0x18 => system.sound_timer = system.v[register],
                0x1E => system.i = system.i + system.v[register] as u16,
                0x29 => {
                    system.i = (system.v[register] * 5) as u16;
                }
                0x33 => {
                    system.mem[system.i as usize] = system.v[register] / 100;
                    system.mem[(system.i + 1) as usize] = (system.v[register] / 10) % 10;
                    system.mem[(system.i + 2) as usize] = (system.v[register] % 100) % 10;
                }
                0x55 => {
                    for read_register in 0..register {
                        system.mem[system.i as usize + read_register] = system.v[read_register];
                    }
                    system.i = system.i + register as u16 + 1
                }
                0x65 => {
                    for read_register in 0..register {
                        system.v[read_register] = system.mem[system.i as usize + read_register];
                    }
                    system.i = system.i + register as u16 + 1
                }
                _ => {}
            }
        },
    ];

    /// Splits a `u16` into an array of 4 words (4-bits) represented as `u8`
    ///
    /// # Arguments
    ///
    /// * `op` - The opcode to split
    fn get_op_words(op: u16) -> [u8; 4] {
        [
            ((op & 0xF000) >> 12) as u8,
            ((op & 0x0F00) >> 8) as u8,
            ((op & 0x00F0) >> 4) as u8,
            (op & 0x000F) as u8,
        ]
    }

    /// Combines two words (4-bits) represented as individual `u8`s into a single `u8`
    ///
    /// # Arguments
    ///
    /// * `first` - The most significant word in the new byte
    /// * `second` - The least significant word in the new byte
    fn combine_words(first: u8, second: u8) -> u8 {
        first << 4 | second
    }

    /// Returns the most significant word from an opcode
    ///
    /// # Arguments
    ///
    /// * `op` - The opcode to modify
    pub fn get_op_group(op: u16) -> u16 {
        (op & 0xF000) >> 12
    }

    /// Splits an opcode into two bytes
    ///
    /// # Arguments
    ///
    /// * `op` - The opcode to split into two bytes
    pub fn split_op(op: u16) -> (u8, u8) {
        ((op >> 8) as u8, (op & 0x00FF) as u8)
    }

    #[cfg(test)]
    mod tests {
        use super::System;

        /// Builds a new system containing the specified ROM memory
        ///
        /// # Arguments
        ///
        /// * `mem` - The ROM memory containing the desired op codes
        fn build_system(mem: Vec<u8>) -> System {
            let mut system = System::new();
            system.write_rom(mem);

            system
        }

        #[test]
        fn test_00ee() {
            let mut system = build_system(vec![0x22, 0x04, 0x00, 0x00, 0x00, 0xEE]);

            system.tick();
            assert_eq!(system.stack[system.sp], 0x202);
            assert_eq!(system.pc, 0x204);
            system.tick();
            assert_eq!(system.pc, 0x202);
        }

        #[test]
        fn test_1000() {
            let mut system = build_system(vec![0x12, 0x04]);

            system.tick();
            assert_eq!(system.pc, 0x204);
        }

        #[test]
        fn test_2000() {
            let mut system = build_system(vec![0x22, 0x04]);

            system.tick();
            assert_eq!(system.sp, 1);
            assert_eq!(system.stack[system.sp], 0x202);
            assert_eq!(system.pc, 0x204);
        }

        #[test]
        fn test_3000_skip() {
            let mut system = build_system(vec![0x30, 0x00]);

            system.tick();
            assert_eq!(system.pc, 0x204);
        }

        #[test]
        fn test_3000_no_skip() {
            let mut system = build_system(vec![0x30, 0x01]);

            system.tick();
            assert_eq!(system.pc, 0x202);
        }

        #[test]
        fn test_4000_skip() {
            let mut system = build_system(vec![0x40, 0x01]);

            system.tick();
            assert_eq!(system.pc, 0x204);
        }

        #[test]
        fn test_4000_no_skip() {
            let mut system = build_system(vec![0x40, 0x00]);

            system.tick();
            assert_eq!(system.pc, 0x202);
        }

        #[test]
        fn test_5000_skip() {
            let mut system = build_system(vec![0x50, 0x00]);

            system.tick();
            assert_eq!(system.pc, 0x204);
        }

        #[test]
        fn test_5000_no_skip() {
            let mut system = build_system(vec![0x50, 0x10]);
            system.v[0x1] = 1;

            system.tick();
            assert_eq!(system.pc, 0x202);
        }

        #[test]
        fn test_6000() {
            let mut system = build_system(vec![
                0x60, 0xF0, 0x61, 0xFF, 0x62, 0xF0, 0x63, 0xFF, 0x64, 0xF0, 0x65, 0xFF, 0x66, 0xF0,
                0x67, 0xFF, 0x68, 0xF0, 0x69, 0xFF, 0x6A, 0xF0, 0x6B, 0xFF, 0x6C, 0xF0, 0x6D, 0xFF,
                0x6E, 0xF0, 0x6F, 0xFF,
            ]);

            // Simulate all 16 opcodes to set registers
            for _ in 0..16 {
                system.tick();
            }

            // Check all 16 registers for alternating values (0xF0, 0xFF)
            for i in 0..8 {
                assert_eq!(system.v[i * 2], 0xF0);
                assert_eq!(system.v[(i * 2) + 1], 0xFF);
            }
        }

        #[test]
        fn test_7000() {
            let mut system = build_system(vec!(
                0x70,
                0xFF,

                0x71,
                0x01,
            ));
            system.v[0x1] = 0xFF;

            system.tick();
            assert_eq!(system.v[0x0], 0xFF);
            system.tick();
            assert_eq!(system.v[0x1], 0x00);
        }
    }
}
