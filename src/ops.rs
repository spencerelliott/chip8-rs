use crate::system::System;

pub const OP_TREE: [fn(&mut System, u16); 16] = [
    |system, op| {  // 0x0XXX
        let (_, inst) = split_op(op);

        if inst == 0xE0 {
            // Clear the screen
        } else if inst == 0xEE {
            // Return from subroutine
            system.pc = system.stack[system.sp];
            system.sp = system.sp - 1;
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

fn get_op_words(op: u16) -> [u8; 4] {
    [
        ((op & 0xF000) >> 12) as u8,
        ((op & 0x0F00) >> 8) as u8,
        ((op & 0x00F0) >> 4) as u8,
        (op & 0x000F) as u8,
    ]
}

fn combine_words(first: u8, second: u8) -> u8 {
    first << 4 & second
}

pub fn get_op_code(op: u16) -> u16 {
    (op & 0xF000) >> 12
}

pub fn split_op(op: u16) -> (u8, u8) {
    ((op >> 8) as u8, (op & 0x00FF) as u8)
}