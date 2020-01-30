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
        system.sp = op as usize;
    },
    |system, op| {  // 0x2XXX
        // Put the program counter on the stack and jump
        system.sp = system.sp + 1;
        system.stack[system.sp] = system.pc;
        system.pc = op as usize;
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

    },
    |system, op| {  // 0x7XXX

    },
    |system, op| {  // 0x8XXX

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

    },
    |system, op| {  // 0xBXXX

    },
    |system, op| {  // 0xCXXX

    },
    |system, op| {  // 0xDXXX

    },
    |system, op| {  // 0xEXXX

    },
    |system, op| {  // 0xFXXX

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
    first << 4 & second
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