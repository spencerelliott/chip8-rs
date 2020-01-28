use crate::ops::{get_op_code, CallOpGroup, JumpOpGroup, LogicOpGroup, OpGroup, SystemOpGroup};

pub struct System {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: usize,
    pub sp: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [usize; 16],
    pub mem: [u8; 4096],
    pub vmem: [u8; 64 * 32],
}

impl System {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            mem: [0; 4096],
            vmem: [0; 64 * 32],
        }
    }

    pub fn write_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.mem[0x200+i] = rom[i];
        }
    }

    pub fn tick(&mut self) {
        let op = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;
        println!("PC: {:X} - op: {:X}", self.pc, op);

        self.execute_op(op);
        self.pc = self.pc + 2;
    }

    fn execute_op(&mut self, op: u16) {
        let code = get_op_code(op);

        match code {
            0x0 => SystemOpGroup::execute(self, op),
            0x1 => JumpOpGroup::execute(self, op),
            0x2 => CallOpGroup::execute(self, op),
            0x3..=0x5 => LogicOpGroup::execute(self, op),
            0x9 => LogicOpGroup::execute(self, op),
            _ => println!("Unknown op code: {:X}", op)
        }
    }
}
