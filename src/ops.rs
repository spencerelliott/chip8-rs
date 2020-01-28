use crate::system::System;

pub trait OpGroup {
    fn execute( system: &mut System, op: u16);
}

pub struct SystemOpGroup;
pub struct JumpOpGroup;
pub struct CallOpGroup;
pub struct LogicOpGroup;

impl OpGroup for SystemOpGroup {
    fn execute(system: &mut System, op: u16) {
        let (_, inst) = split_op(op);

        if inst == 0xE0 {
            // Clear the screen
        } else if inst == 0xEE {
            // Return from subroutine
            system.pc = system.stack[system.sp];
            system.sp = system.sp - 1;
        }
    }
}

impl OpGroup for JumpOpGroup {
    fn execute(system: &mut System, op: u16) {
        // Jump program counter
        system.pc = op as usize;
    }
}

impl OpGroup for CallOpGroup {
    fn execute(system: &mut System, op: u16) {
        // Put the program counter on the stack and jump
        system.sp = system.sp + 1;
        system.stack[system.sp] = system.pc;
        system.pc = op as usize;
    }
}

impl OpGroup for LogicOpGroup {
    fn execute(system: &mut System, op: u16) {
        let words = get_op_words(op);
        let register = words[1] as usize;

        match get_op_code(op) {
            0x3 => {
                let value = combine_words(words[2], words[3]);

                if system.v[register] == value {
                    system.pc = system.pc + 2;
                }
            }
            0x4 => {
                let value = combine_words(words[2], words[3]);

                if system.v[register] != value {
                    system.pc = system.pc + 2;
                }
            }
            0x5 => {
                let cmp_register = words[2] as usize;

                if system.v[register] == system.v[cmp_register] {
                    system.pc = system.pc + 2;
                }
            }
            0x9 => {
                let cmp_register = words[2] as usize;

                if system.v[register] != system.v[cmp_register] {
                    system.pc = system.pc + 2;
                }
            }
            _ => { }
        }
    }
}

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