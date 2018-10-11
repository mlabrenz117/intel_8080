use crate::instruction::{Instruction, Opcode, OpcodeSize};
use std::fmt::{self, Display};

pub(super) struct ProgramCounter<'a> {
    pub(super) addr: u16,
    rom: &'a [u8],
}

impl<'a> ProgramCounter<'a> {
    pub(super) fn new(rom: &'a [u8]) -> Self {
        ProgramCounter { addr: 0, rom }
    }

    pub(super) fn addr(&self) -> usize {
        self.addr as usize
    }
}

impl<'a> Iterator for ProgramCounter<'a> {
    type Item = Instruction;
    fn next(&mut self) -> Option<Self::Item> {
        if self.addr() >= self.rom.len() {
            return None;
        }
        let opcode = Opcode::from(self.rom[self.addr()]);
        self.addr += 1;
        match opcode.size() {
            OpcodeSize::Binary => {
                let data: u8 = self.rom[self.addr()];
                self.addr += 1;
                Some(Instruction::new_binary(opcode, data))
            }
            OpcodeSize::Trinary => {
                let data_low = self.rom[self.addr()] as u16;
                let data_high = self.rom[self.addr() + 1] as u16;
                let data_high = data_high << 8;
                let addr: u16 = data_high | data_low;
                self.addr += 2;
                Some(Instruction::new_trinary(opcode, addr))
            }
            OpcodeSize::Unary => Some(Instruction::new_unary(opcode)),
        }
    }
}

impl<'a> Display for ProgramCounter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x}", self.addr)
    }
}
