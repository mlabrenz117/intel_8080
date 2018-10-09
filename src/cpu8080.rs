use crate::disassembler::Disassembler;
use crate::disassembler::{Instruction, Opcode};
use failure::Error;

mod arithmetic;
mod branch;
mod data_transfer;
mod io;
mod logical;
mod special;
mod stack;

pub struct Cpu8080<'a> {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    disassembler: Disassembler<'a>,
    memory: [u8; 8000],
    flags: ConditionalFlags,
    int_enable: u8,
}

impl<'a> Cpu8080<'a> {
    pub fn new(buf: &'a [u8]) -> Cpu8080 {
        Cpu8080 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            disassembler: Disassembler::new(buf),
            memory: [0; 8000],
            flags: ConditionalFlags::new(),
            int_enable: 1,
        }
    }

    fn emulate_instruction(&mut self, instruction: Instruction) -> Result<(), Error> {
        use self::Opcode::*;
        match instruction.opcode() {
            NOP => Ok(()),
            LXI(r) => {
                let params = instruction
                    .trinary_params()
                    .expect("LXI should always have 2 param bytes");
                self.lxi(r, params)
            }
            LDAX(r) => self.ldax(r),
            INX(r) => self.inx(r),
            DCR(r) => self.dcr(r),
            MOV(d, s) => self.mov(d, s),
            MVI(r) => {
                let param = instruction
                    .binary_params()
                    .expect("MVI param should be 1 byte");
                self.mvi(r, param)
            }
            ADD(r) => self.add(r),
            ADI => {
                let param = instruction
                    .binary_params()
                    .expect("ADI should always have 1 param byte");
                self.adi(param)
            }
            SUB(r) => self.sub(r),
            SUI => {
                let param = instruction
                    .binary_params()
                    .expect("SUI should always have 1 param byte");
                self.sui(param)
            }
            JMP => {
                let addr = instruction
                    .trinary_params()
                    .expect("JMP Address should have 2 bytes");
                self.jmp(addr)
            }
            CALL => {
                let addr = instruction
                    .trinary_params()
                    .expect("CALL Address should have 2 bytes");
                self.call(addr)
            }
            _op => bail!("Instruction not yet implemented: {:?}", _op),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        while let Some(instruction) = self.disassembler.next() {
            println!("Executing Instruction: {}", instruction);
            self.emulate_instruction(instruction)?
        }
        Ok(())
    }

    fn set_8bit_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
            Register::M => {
                self.l = value;
                self.h = 0
            }
            Register::SP => self.sp = value as u16,
        };
    }

    fn get_8bit_register(&self, register: Register) -> Result<u8, Error> {
        match register {
            Register::A => Ok(self.a),
            Register::B => Ok(self.b),
            Register::C => Ok(self.c),
            Register::D => Ok(self.d),
            Register::E => Ok(self.e),
            Register::H => Ok(self.h),
            Register::L => Ok(self.l),
            _r => bail!("{:?} is not 8 bit"),
        }
    }

    fn set_mem_val(&mut self, value: u8) {
        self.set_mem_loc(self.m(), value);
    }

    fn get_mem_val(&self) -> u8 {
        self.get_mem_loc(self.m())
    }

    fn m(&self) -> u16 {
        let high = self.get_8bit_register(Register::H).unwrap() as u16;
        let low = self.get_8bit_register(Register::L).unwrap() as u16;
        high << 8 | low
    }

    fn set_sp_register(&mut self, value: u16) {
        self.sp = value;
    }

    fn get_sp_register(&self) -> u16 {
        self.sp
    }

    fn push_u16(&mut self, value: u16) -> Result<(), Error> {
        let loc_low = self.sp - 2;
        let loc_high = self.sp - 1;
        let (high, low) = split_bytes(value);
        if loc_low < 0x2000 {
            bail!("Stack Overflow")
        };
        self.set_mem_loc(loc_low, low);
        self.set_mem_loc(loc_high, high);
        self.sp -= 2;
        Ok(())
    }

    fn push_u8(&mut self, value: u8) -> Result<(), Error> {
        let loc = self.sp - 1;
        if loc < 0x2000 {
            bail!("Stack Overflow")
        };
        self.set_mem_loc(loc, value);
        self.sp -= 1;
        Ok(())
    }

    fn get_mem_loc(&self, addr: u16) -> u8 {
        match addr < 0x2000 {
            true => self.disassembler.value_at(addr),
            false => {
                let addr = addr - 0x2000;
                self.memory[addr as usize]
            }
        }
    }

    fn set_mem_loc(&mut self, location: u16, value: u8) {
        let location = location - 0x2000;
        self.memory[location as usize] = value;
        //std::mem::replace(&mut self.memory[location as usize], value);
    }
}

pub fn split_bytes(bytes: u16) -> (u8, u8) {
    let low_byte = (bytes & 0x00ff) as u8;
    let high_byte = (bytes & 0xff00) >> 8;
    let high_byte = high_byte as u8;
    (high_byte, low_byte)
}

pub fn concat_bytes(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | (low as u16)
}

pub fn check_parity(num: u8) -> bool {
    let mut bytes = num;
    let mut parity = 0;
    while bytes > 0 {
        parity ^= bytes % 2;
        bytes >>= 1;
    }
    parity == 0
}

#[derive(Debug)]
struct ConditionalFlags {
    z: bool,
    s: bool,
    p: bool,
    cy: bool,
    ac: bool,
    pad: u8,
}

impl ConditionalFlags {
    fn new() -> ConditionalFlags {
        ConditionalFlags {
            z: false,
            s: false,
            p: false,
            cy: false,
            ac: false,
            pad: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    SP,
}

impl Register {
    pub fn get_pair(&self) -> Option<Register> {
        match self {
            Register::B => Some(Register::C),
            Register::D => Some(Register::E),
            Register::H => Some(Register::L),
            _ => None,
        }
    }

    pub fn is_8bit(&self) -> bool {
        match self {
            Register::A => true,
            Register::B => true,
            Register::C => true,
            Register::D => true,
            Register::E => true,
            Register::H => true,
            Register::L => true,
            Register::M => false,
            Register::SP => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_parity, concat_bytes, split_bytes};
    #[test]
    fn can_split_bytes() {
        let (high, low) = split_bytes(0xea14);
        assert_eq!(high, 0xea);
        assert_eq!(low, 0x14);
    }

    #[test]
    fn can_concat_bytes() {
        let low = 0x14;
        let high = 0xea;
        assert_eq!(concat_bytes(high, low), 0xea14);
    }

    #[test]
    fn can_test_parity() {
        let odd = 0x5b; // 91
        assert_eq!(check_parity(odd), false);
        let even = 0x9f; // 159
        assert_eq!(check_parity(even), true);
    }
}
