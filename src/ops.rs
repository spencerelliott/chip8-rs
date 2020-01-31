use crate::system::System;

/// All avaialable opcodes where the most-significant word (`0x0XXX` - `0xFXXX`) is the index. CHIP-8
/// uses 16 bits for op codes. The most significant word (4-bits) is generally used to define the operation
/// performed while the next 3 bytes are the "parameters".
/// 
/// Each group (item within this list) handles all opcodes within the range of that most significant bit. For 
/// example, the `0x0` group will handle any opcodes ranging from `0x0000` to `0x0FFF`, if they exist. In the
/// case of CHIP-8, the only two opcodes in the `0x0` group are `0x0E0` and `0x0EE`.
pub const OP_TREE: [fn(&mut System, u16); 16] = [
    |system, op| {  // 0x0XXX
        match split_op(op).1 {
            0xE0 => {}
            0xEE => {
                // Return from subroutine
                system.pc = system.stack[system.sp];
                system.sp = system.sp - 1;
            }
            _ => {}
        }
    },
    |system, op| {  // 0x1XXX
        system.sp = (op & 0x0FFF) as usize;
    },
    |system, op| {  // 0x2XXX
        // Put the program counter on the stack and jump
        system.sp = system.sp + 1;
        system.stack[system.sp] = system.pc;
        system.pc = (op & 0x0FFF) as usize;
    },
    |system, op| {  // 0x3XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let value = combine_words(words[2], words[3]);

        if system.v[register] == value {
            system.pc = system.pc + 2;
        }
    },
    |system, op| {  // 0x4XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let value = combine_words(words[2], words[3]);

        if system.v[register] != value {
            system.pc = system.pc + 2;
        }
    },
    |system, op| {  // 0x5XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let cmp_register = words[2] as usize;

        if system.v[register] == system.v[cmp_register] {
            system.pc = system.pc + 2;
        }
    },
    |system, op| {  // 0x6XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let value = combine_words(words[2], words[3]);

        system.v[register] = value;
    },
    |system, op| {  // 0x7XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let value = combine_words(words[2], words[3]);

        system.v[register] = system.v[register] + value;
    },
    |system, op| {  // 0x8XXX
        let words = get_op_words(op);
        let register1 = words[1] as usize;
        let register2 = words[2] as usize;

        match words[3] {
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
                let (value, overflow) = system.v[register1].overflowing_add(system.v[register2]);

                system.v[0xF] = if overflow { 1 } else { 0 };
                system.v[register1] = value;
            }
            0x5 => {
                let (value, overflow) = system.v[register1].overflowing_sub(system.v[register2]);

                system.v[0xF] = if overflow { 1 } else { 0 };
                system.v[register1] = value;
            }
            0x6 => {
                let (value, overflow) = system.v[register1].overflowing_shr(1);

                system.v[0xF] = if overflow { 1 } else { 0 };
                system.v[register1] = value;
            }
            0x7 => {
                let (value, overflow) = system.v[register2].overflowing_sub(system.v[register1]);

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
    |system, op| {  // 0x9XXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let cmp_register = words[2] as usize;

        if system.v[register] != system.v[cmp_register] {
            system.pc = system.pc + 2;
        }
    },
    |system, op| {  // 0xAXXX
        system.i = (op & 0x0FFF) as u16;
    },
    |system, op| {  // 0xBXXX
        system.pc = (system.v[0x0] + (op & 0x0FFF) as u8) as usize;
    },
    |system, op| {  // 0xCXXX

    },
    |system, op| {  // 0xDXXX

    },
    |system, op| {  // 0xEXXX

    },
    |system, op| {  // 0xFXXX
        let words = get_op_words(op);
        let register = words[1] as usize;
        let instruction = combine_words(words[2], words[3]);

        match instruction {
            0x07 => system.v[register] = system.delay_timer,
            0x0A => { }
            0x15 => system.delay_timer = system.v[register],
            0x18 => system.sound_timer = system.v[register],
            0x1E => system.i = system.i + system.v[register] as u16,
            0x29 => { }
            0x33 => { }
            0x55 => {
                for read_register in 0..register {
                    system.mem[system.i as usize + read_register] = system.v[read_register];
                }
            }
            0x65 => {
                for read_register in 0..register {
                    system.v[read_register] = system.mem[system.i as usize + read_register];
                }
            }
            _ => { }
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
pub fn get_op_code(op: u16) -> u16 {
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