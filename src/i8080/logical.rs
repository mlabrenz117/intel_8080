use super::{TwosComplement, I8080};
use crate::i8080::{error::EmulateError, Register, Result};
use crate::instruction::{InstructionData, Opcode};
use crate::interconnect::Interconnect;

impl I8080 {
    pub(super) fn cpi(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let (v, c) = self.a.complement_sub(value);
            self.flags.set_non_carry_flags(v);
            self.flags.cy = c;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::CPI,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn ani(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let result = self.get_8bit_register(Register::A)? & value;
            self.set_8bit_register(Register::A, result);
            self.flags.set_non_carry_flags(result);
            self.flags.cy = false;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::ANI,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn ana(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        let value: u8 = match register {
            Register::SP => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::ANA(register),
                    register,
                })
            }
            Register::M => interconnect.read_byte(self.m()),
            _r => self.get_8bit_register(_r)?,
        };
        let result = self.a & value;
        self.flags.set_non_carry_flags(result);
        self.flags.cy = false;
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(super) fn xra(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        let value: u8 = match register {
            Register::SP => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::XRA(register),
                    register,
                })
            }
            Register::M => interconnect.read_byte(self.m()),
            _r => self.get_8bit_register(_r)?,
        };
        let result = self.a ^ value;
        self.flags.set_non_carry_flags(result);
        self.flags.cy = false;
        self.set_8bit_register(Register::A, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu8080::Cpu8080;
    #[test]
    fn cpi() {
        let bytecode = [
            0xfe, 0x6f, // CPI 0x6f
            0xfe, 0x5f, // CPI 0x5f
            0xfe, 0x4f, // CPI 0x4f
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x5f;
        cpu.step();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);
        assert_eq!(cpu.flags.cy, true);
        cpu.step();
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
        cpu.step();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
    }

    #[test]
    fn ani() {
        let bytecode = [
            0xe6, 0x0f, // ANI 0x0f
            0xe6, 0x22, // ANI 0x22
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x3a;
        cpu.step();
        assert_eq!(cpu.a, 0x0a);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.s, false);
        cpu.a = 0x69;
        cpu.step();
        assert_eq!(cpu.a, 0x20);
    }

    #[test]
    fn ana() {
        let bytecode = [
            0xa5, // ANA L
            0xa6, // ANA M
            0xa7, // ANA A
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x0a;
        cpu.h = 0x20;
        cpu.l = 0xc5;
        cpu.write_memory(0x20c5, 0xd4).unwrap();
        cpu.step();
        assert_eq!(cpu.a, 0x00);
        cpu.a = 0xff;
        cpu.step();
        assert_eq!(cpu.a, 0xd4);
        cpu.step();
        assert_eq!(cpu.a, 0xd4);
    }

    #[test]
    fn xra() {
        let bytecode = [
            0xad, // XRA L
            0xae, // XRA M
            0xaf, // XRA A
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x0a;
        cpu.h = 0x20;
        cpu.l = 0xc5;
        cpu.write_memory(0x20c5, 0xd4).unwrap();
        cpu.step();
        assert_eq!(cpu.a, 0xcf);
        cpu.step();
        assert_eq!(cpu.a, 0x1b);
        cpu.step();
        assert_eq!(cpu.a, 0x00);
    }
}
