use crate::i8080::*;
use crate::instruction::{InstructionData, Opcode};
use crate::interconnect::Interconnect;

impl I8080 {
    pub(crate) fn cpi(&mut self, data: InstructionData) -> Result<()> {
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

    pub(crate) fn ani(&mut self, data: InstructionData) -> Result<()> {
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

    pub(crate) fn ana(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
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

    pub(crate) fn xra(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
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
    use crate::Emulator;
    #[test]
    fn cpi() {
        let bytecode = [
            0xfe, 0x6f, // CPI 0x6f
            0xfe, 0x5f, // CPI 0x5f
            0xfe, 0x4f, // CPI 0x4f
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x5f;
        system.step();
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, true);
        assert_eq!(system.cpu.flags.cy, true);
        system.step();
        assert_eq!(system.cpu.flags.z, true);
        assert_eq!(system.cpu.flags.s, false);
        assert_eq!(system.cpu.flags.cy, false);
        system.step();
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, false);
        assert_eq!(system.cpu.flags.cy, false);
    }

    #[test]
    fn ani() {
        let bytecode = [
            0xe6, 0x0f, // ANI 0x0f
            0xe6, 0x22, // ANI 0x22
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x3a;
        system.step();
        assert_eq!(system.cpu.a, 0x0a);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.s, false);
        system.cpu.a = 0x69;
        system.step();
        assert_eq!(system.cpu.a, 0x20);
    }

    #[test]
    fn ana() {
        let bytecode = [
            0xa5, // ANA L
            0xa6, // ANA M
            0xa7, // ANA A
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x0a;
        system.cpu.h = 0x20;
        system.cpu.l = 0xc5;
        system.interconnect.write_byte(0x20c5, 0xd4);
        system.step();
        assert_eq!(system.cpu.a, 0x00);
        system.cpu.a = 0xff;
        system.step();
        assert_eq!(system.cpu.a, 0xd4);
        system.step();
        assert_eq!(system.cpu.a, 0xd4);
    }

    #[test]
    fn xra() {
        let bytecode = [
            0xad, // XRA L
            0xae, // XRA M
            0xaf, // XRA A
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x0a;
        system.cpu.h = 0x20;
        system.cpu.l = 0xc5;
        system.interconnect.write_byte(0x20c5, 0xd4);
        system.step();
        assert_eq!(system.cpu.a, 0xcf);
        system.step();
        assert_eq!(system.cpu.a, 0x1b);
        system.step();
        assert_eq!(system.cpu.a, 0x00);
    }
}
