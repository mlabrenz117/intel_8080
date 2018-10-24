#[macro_use]
extern crate failure;

pub(crate) mod mem_map;

pub mod game_pad;
pub mod i8080;
pub mod instruction;
pub mod interconnect;
pub mod rom;
pub mod vram;
pub mod wram;

use log::error;

use self::instruction::*;
use self::rom::Rom;

pub struct System {
    cpu: i8080::I8080,
    interconnect: interconnect::Interconnect,
}

impl System {
    pub fn new(rom: Rom) -> System {
        System {
            cpu: i8080::I8080::new(),
            interconnect: interconnect::Interconnect::new(rom),
        }
    }

    pub fn step(&mut self) {
        if let Some(instruction) = self.next_instruction() {
            if let Err(e) = self
                .cpu
                .emulate_instruction(instruction, &mut self.interconnect)
            {
                error!("{}", e);
            }
        }
    }

    pub fn run(&mut self) {
        while let Some(instruction) = self.next_instruction() {
            if let Err(e) = self
                .cpu
                .emulate_instruction(instruction, &mut self.interconnect)
            {
                error!("{}", e);
                break;
            }
        }
    }

    fn next_instruction(&self) -> Option<Instruction> {
        if (self.cpu.pc() as usize) >= self.interconnect.rom_len() {
            None
        } else {
            let opcode = Opcode::from(self.interconnect.read_byte(self.cpu.pc()));
            let instruction = match opcode.size() {
                OpcodeSize::Binary => {
                    let data = self.interconnect.read_byte(self.cpu.pc() + 1);
                    Instruction::new_binary(opcode, data).unwrap()
                }
                OpcodeSize::Trinary => {
                    let data_low = self.interconnect.read_byte(self.cpu.pc() + 1) as u16;
                    let data_high = self.interconnect.read_byte(self.cpu.pc() + 2) as u16;
                    let addr = (data_high << 8) | data_low;
                    Instruction::new_trinary(opcode, addr).unwrap()
                }
                OpcodeSize::Unary => Instruction::new_unary(opcode).unwrap(),
            };
            Some(instruction)
        }
    }
}
