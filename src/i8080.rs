use crate::instruction::{Instruction, Opcode};
use crate::interconnect::Interconnect;
use log::info;
use std::fmt::{self, Display};

mod flags;
pub use self::flags::ConditionalFlags;

mod register;
pub use self::register::Register;

mod error;
use self::error::EmulateError;

type Result<T> = std::result::Result<T, EmulateError>;

// Instruction Implementations
mod implementations;

pub struct I8080 {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    flags: ConditionalFlags,
    rc: [bool; 8],
    interrupts_enabled: bool,
}

impl I8080 {
    pub fn new() -> I8080 {
        I8080 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            flags: ConditionalFlags::new(),
            rc: [false; 8],
            interrupts_enabled: true,
        }
    }

    pub fn emulate_instruction(
        &mut self,
        instruction: Instruction,
        ic: &mut Interconnect,
    ) -> Result<()> {
        let old_pc = self.pc;
        self.pc += instruction.len();
        use self::Opcode::*;
        self.reset_rc();
        let r = match instruction.opcode() {
            NOP => Ok(()),
            // Data transfer Instructions
            LXI(r) => self.lxi(r, instruction.data()),
            LDAX(r) => self.ldax(r, ic),
            LDA => self.lda(instruction.data(), ic),
            STA => self.sta(instruction.data(), ic),
            MOV(d, s) => self.mov(d, s, ic),
            MVI(r) => self.mvi(r, instruction.data(), ic),
            XCHG => self.xchg(),
            PUSH(r) => self.push(r, ic),
            POP(r) => self.pop(r, ic),
            // Arithmetic Instructions
            INX(r) => self.inx(r),
            DCR(r) => self.dcr(r, ic),
            ADD(r) => self.add(r, ic),
            ADI => self.adi(instruction.data()),
            DAD(r) => self.dad(r),
            SUB(r) => self.sub(r, ic),
            SUI => self.sui(instruction.data()),
            RRC => self.rrc(),
            // Logical Instructions
            CPI => self.cpi(instruction.data()),
            ANI => self.ani(instruction.data()),
            ANA(r) => self.ana(r, ic),
            XRA(r) => self.xra(r, ic),
            // IO Instructions
            OUT => self.out(instruction.data()),
            // Branch Instructions
            JMP => self.jmp(instruction.data()),
            JNZ => self.jnz(instruction.data()),
            CALL => self.call(instruction.data(), ic),
            RET => self.ret(ic),
            // Special Instructions
            EI => self.ei(),
            _op => return Err(EmulateError::UnimplementedInstruction { instruction }),
        };

        if let Ok(()) = r {
            info!("{}: {}; {}", old_pc, instruction, self);
        }
        r
    }

    fn set_8bit_register(&mut self, register: Register, value: u8) {
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

    pub fn get_8bit_register(&self, register: Register) -> Result<u8> {
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

    fn set_m(&mut self, addr: u16) {
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
        self.pc
    }

    pub fn flags(&self) -> ConditionalFlags {
        self.flags
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.interrupts_enabled
    }

    fn push_u16(&mut self, value: u16, interconnect: &mut Interconnect) -> Result<()> {
        let (high, low) = split_bytes(value);
        self.push_u8(high, interconnect)?;
        self.push_u8(low, interconnect)?;
        Ok(())
    }

    fn push_u8(&mut self, value: u8, interconnect: &mut Interconnect) -> Result<()> {
        let loc = self.sp - 1;
        if loc < 0x2000 {
            return Err(EmulateError::StackOverflow);
        };
        interconnect.write_byte(loc, value);
        self.sp -= 1;
        self.register_changed(Register::SP);
        Ok(())
    }

    fn pop_u8(&mut self, interconnect: &Interconnect) -> Result<u8> {
        let value = interconnect.read_byte(self.sp);
        self.sp += 1;
        self.register_changed(Register::SP);
        Ok(value)
    }

    fn pop_u16(&mut self, interconnect: &Interconnect) -> Result<u16> {
        let low = self.pop_u8(interconnect)?;
        let high = self.pop_u8(interconnect)?;
        Ok(concat_bytes(high, low))
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

impl Display for I8080 {
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
    use super::{concat_bytes, split_bytes};
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
}
