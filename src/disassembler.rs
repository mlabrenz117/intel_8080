pub mod instruction;
pub use self::instruction::{Instruction, Opcode, OpcodeSize};

#[derive(Debug)]
pub struct Disassembler<'a> {
    pc: usize,
    buf: &'a [u8],
}

impl<'a> Disassembler<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Disassembler { pc: 0, buf }
    }
}

impl<'a> Disassembler<'a> {
    pub(super) fn update_pc(&mut self, addr: u16) {
        self.pc = addr as usize;
    }

    pub(super) fn pc(&self) -> u16 {
        self.pc as u16
    }

    pub(super) fn value_at(&self, addr: u16) -> u8 {
        self.buf[addr as usize]
    }
}

impl<'a> Iterator for Disassembler<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pc >= self.buf.len() {
            return None;
        }
        let opcode = Opcode::from(self.buf[self.pc]);
        match opcode.size() {
            OpcodeSize::Binary => {
                self.pc += 1;
                let data: u8 = self.buf[self.pc];
                self.pc += 1;
                Some(Instruction::new_binary(opcode, data))
            }
            OpcodeSize::Trinary => {
                self.pc += 1;
                let data_low: u16 = self.buf[self.pc] as u16;
                self.pc += 1;
                let data_high: u16 = self.buf[self.pc] as u16;
                let data_high = data_high << 8;
                self.pc += 1;
                let data: u16 = data_high + data_low;
                Some(Instruction::new_trinary(opcode, data))
            }
            OpcodeSize::Unary => {
                self.pc += 1;
                Some(Instruction::new_unary(opcode))
            }
        }
    }
}
