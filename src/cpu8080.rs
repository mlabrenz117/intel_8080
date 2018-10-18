use crate::instruction::{Instruction, Opcode};
use log::{error, info};
use std::fmt::{self, Display};

mod program_counter;
use self::program_counter::ProgramCounter;

mod error;
use self::error::EmulateError;

// Instruction Implementations
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
    pc: ProgramCounter<'a>,
    rom: &'a [u8],
    memory: [u8; 0xffff],
    devices: [u8; 0xff],
    flags: ConditionalFlags,
    rc: [bool; 8],
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
            pc: ProgramCounter::new(buf),
            rom: buf,
            memory: [0; 0xffff],
            devices: [0; 0xff],
            flags: ConditionalFlags::new(),
            //int_enable: 1,
            rc: [false; 8],
        }
    }

    pub fn emulate_instruction(&mut self, instruction: Instruction) -> Result<(), EmulateError> {
        use self::Opcode::*;
        self.reset_rc();
        match instruction.opcode() {
            NOP => Ok(()),
            // Data transfer Instructions
            LXI(r) => self.lxi(r, instruction.data()),
            LDAX(r) => self.ldax(r),
            MOV(d, s) => self.mov(d, s),
            MVI(r) => self.mvi(r, instruction.data()),
            XCHG => self.xchg(),
            PUSH(r) => self.push(r),
            POP(r) => self.pop(r),
            // Arithmetic Instructions
            INX(r) => self.inx(r),
            DCR(r) => self.dcr(r),
            ADD(r) => self.add(r),
            ADI => self.adi(instruction.data()),
            DAD(r) => self.dad(r),
            SUB(r) => self.sub(r),
            SUI => self.sui(instruction.data()),
            CPI => self.cpi(instruction.data()),
            RRC => self.rrc(),
            // IO Instructions
            OUT => self.out(instruction.data()),
            // Branch Instructions
            JMP => self.jmp(instruction.data()),
            JNZ => self.jnz(instruction.data()),
            CALL => self.call(instruction.data()),
            RET => self.ret(),
            _op => return Err(EmulateError::UnimplementedInstruction { instruction }),
        }
    }

    pub fn run(&mut self) {
        while let Some(instruction) = self.pc.next() {
            if let Err(e) = self.emulate_instruction(instruction) {
                error!("{}", e);
                break;
            } else {
                info!("{}: {}; {}", self.pc, instruction, self);
            }
        }
    }

    pub fn step(&mut self) {
        if let Some(instruction) = self.pc.next() {
            if let Err(e) = self.emulate_instruction(instruction) {
                error!("{}", e);
            } else {
                info!("{}: {}{}", self.pc, instruction, self);
            }
        }
    }

    pub fn set_8bit_register(&mut self, register: Register, value: u8) {
        self.register_changed(register);
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

    pub fn get_8bit_register(&self, register: Register) -> Result<u8, EmulateError> {
        match register {
            Register::A => Ok(self.a),
            Register::B => Ok(self.b),
            Register::C => Ok(self.c),
            Register::D => Ok(self.d),
            Register::E => Ok(self.e),
            Register::H => Ok(self.h),
            Register::L => Ok(self.l),
            _r => return Err(EmulateError::RegisterNot8Bit { register }),
        }
    }

    pub fn m(&self) -> u16 {
        let high = self.get_8bit_register(Register::H).unwrap() as u16;
        let low = self.get_8bit_register(Register::L).unwrap() as u16;
        high << 8 | low
    }

    pub fn set_m(&mut self, addr: u16) {
        let (high, low) = split_bytes(addr);
        self.set_8bit_register(Register::H, high);
        self.set_8bit_register(Register::L, low);
    }

    fn set_sp(&mut self, value: u16) {
        self.register_changed(Register::SP);
        self.sp = value;
    }

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn pc(&self) -> u16 {
        self.pc.addr
    }

    pub fn set_pc(&mut self, addr: u16) {
        self.pc.addr = addr;
    }

    pub fn push_u16(&mut self, value: u16) -> Result<(), EmulateError> {
        let loc_low = self.sp - 2;
        let loc_high = self.sp - 1;
        let (high, low) = split_bytes(value);
        if loc_low < 0x2000 {
            return Err(EmulateError::StackOverflow);
        };
        self.write_memory(loc_low, low)?;
        self.write_memory(loc_high, high)?;
        self.sp -= 2;
        self.register_changed(Register::SP);
        Ok(())
    }

    pub fn push_u8(&mut self, value: u8) -> Result<(), EmulateError> {
        let loc = self.sp - 1;
        if loc < 0x2000 {
            return Err(EmulateError::StackOverflow);
        };
        self.write_memory(loc, value)?;
        self.sp -= 1;
        self.register_changed(Register::SP);
        Ok(())
    }

    pub fn pop_u8(&mut self) -> Result<u8, EmulateError> {
        let value = self.read_memory(self.sp);
        self.sp += 1;
        self.register_changed(Register::SP);
        Ok(value)
    }

    pub fn pop_u16(&mut self) -> Result<u16, EmulateError> {
        let low = self.read_memory(self.sp);
        let high = self.read_memory(self.sp + 1);
        self.sp += 2;
        self.register_changed(Register::SP);
        Ok(concat_bytes(high, low))
    }

    pub fn read_memory(&self, addr: u16) -> u8 {
        match addr < 0x2000 {
            true => self.rom[addr as usize],
            false => {
                let addr = addr - 0x2000;
                self.memory[addr as usize]
            }
        }
    }

    pub fn write_memory(&mut self, addr: u16, value: u8) -> Result<(), EmulateError> {
        match addr < 0x2000 {
            true => return Err(EmulateError::WriteToROM),
            false => {
                let addr = addr - 0x2000;
                self.memory[addr as usize] = value;
            }
        }
        Ok(())
    }

    pub fn write_device(&mut self, device: u8, value: u8) {
        self.devices[device as usize] = value;
    }

    pub fn read_device(&self, device: u8) -> u8 {
        self.devices[device as usize]
    }

    fn register_changed(&mut self, reg: Register) {
        match reg {
            Register::A => self.rc[0] = true,
            Register::B => self.rc[1] = true,
            Register::C => self.rc[2] = true,
            Register::D => self.rc[3] = true,
            Register::E => self.rc[4] = true,
            Register::H => self.rc[5] = true,
            Register::L => self.rc[6] = true,
            Register::M => {
                self.rc[5] = true;
                self.rc[6] = true
            }
            Register::SP => self.rc[7] = true,
        }
    }

    fn reset_rc(&mut self) {
        for i in self.rc.iter_mut() {
            *i = false;
        }
    }
}

pub(crate) fn split_bytes(bytes: u16) -> (u8, u8) {
    let low_byte = (bytes & 0x00ff) as u8;
    let high_byte = (bytes & 0xff00) >> 8;
    let high_byte = high_byte as u8;
    (high_byte, low_byte)
}

pub(crate) fn concat_bytes(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | (low as u16)
}

pub(crate) fn check_parity(num: u8) -> bool {
    let mut bytes = num;
    let mut parity = 0;
    while bytes > 0 {
        parity ^= bytes % 2;
        bytes >>= 1;
    }
    parity == 0
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct ConditionalFlags {
    z: bool,
    s: bool,
    p: bool,
    cy: bool,
    ac: bool,
}

impl ConditionalFlags {
    fn new() -> ConditionalFlags {
        ConditionalFlags {
            z: false,
            s: false,
            p: false,
            cy: false,
            ac: false,
        }
    }
}

impl From<ConditionalFlags> for u8 {
    fn from(flag: ConditionalFlags) -> u8 {
        let s = (flag.s as u8) << 7;
        let z = (flag.z as u8) << 6;
        let ac = (flag.ac as u8) << 4;
        let p = (flag.p as u8) << 2;
        let c = flag.cy as u8;
        s | z | ac | p | c | 2
    }
}

impl From<u8> for ConditionalFlags {
    fn from(byte: u8) -> ConditionalFlags {
        ConditionalFlags {
            s: byte & 0x80 != 0x00,
            z: byte & 0x40 != 0x00,
            ac: byte & 0x10 != 0x00,
            p: byte & 0x04 != 0x00,
            cy: byte & 0x01 != 0x00,
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

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Register::A => "A",
            Register::B => "B",
            Register::C => "C",
            Register::D => "D",
            Register::E => "E",
            Register::H => "H",
            Register::L => "L",
            Register::M => "M",
            Register::SP => "S",
        };
        write!(f, "{}", s)
    }
}

trait TwosComplement<RHS = Self> {
    type Output;
    fn complement_sub(self, subtrahend: RHS) -> Self::Output;
}

impl TwosComplement for u8 {
    type Output = (u8, bool);
    fn complement_sub(self, subtrahend: Self) -> Self::Output {
        let complement = !subtrahend + 1;
        let (value, carry) = self.overflowing_add(complement);
        (value, !carry)
    }
}

impl<'a> Display for Cpu8080<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;
        let a = match self.rc[0] {
            true => format!("{:02x}", self.a).blue(),
            false => format!("{:02x}", self.a).white(),
        };
        let b = match self.rc[1] {
            true => format!("{:02x}", self.b).blue(),
            false => format!("{:02x}", self.b).white(),
        };
        let c = match self.rc[2] {
            true => format!("{:02x}", self.c).blue(),
            false => format!("{:02x}", self.c).white(),
        };
        let d = match self.rc[3] {
            true => format!("{:02x}", self.d).blue(),
            false => format!("{:02x}", self.d).white(),
        };
        let e = match self.rc[4] {
            true => format!("{:02x}", self.e).blue(),
            false => format!("{:02x}", self.e).white(),
        };
        let h = match self.rc[5] {
            true => format!("{:02x}", self.h).blue(),
            false => format!("{:02x}", self.h).white(),
        };
        let l = match self.rc[6] {
            true => format!("{:02x}", self.l).blue(),
            false => format!("{:02x}", self.l).white(),
        };
        let s = match self.rc[7] {
            true => format!("{:02x}", self.sp).blue(),
            false => format!("{:02x}", self.sp).white(),
        };
        write!(
            f,
            "CPU: a={}|b={}|c={}|d={}|e={}|h={}|l={}|sp={}",
            a, b, c, d, e, h, l, s,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{check_parity, concat_bytes, split_bytes, ConditionalFlags};
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

    #[test]
    fn flags_as_bytes() {
        let mut f = ConditionalFlags::new();
        assert_eq!(u8::from(f), 0x02);
        f.s = true;
        assert_eq!(u8::from(f), 0x82);
        f.z = true;
        assert_eq!(u8::from(f), 0xc2);
        f.ac = true;
        assert_eq!(u8::from(f), 0xd2);
        f.p = true;
        assert_eq!(u8::from(f), 0xd6);
        f.cy = true;
        assert_eq!(u8::from(f), 0xd7);
    }
}
