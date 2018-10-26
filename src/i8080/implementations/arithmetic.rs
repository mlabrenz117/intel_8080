use crate::i8080::*;
use crate::instruction::{InstructionData, Opcode};
use crate::interconnect::Interconnect;

impl I8080 {
    pub(crate) fn inx(&mut self, register: Register) -> Result<()> {
        if let Some(r2) = register.get_pair() {
            let low = self.get_8bit_register(r2).unwrap();
            let high = self.get_8bit_register(register).unwrap();
            let mut value = concat_bytes(high, low);
            value += 1;
            let (high, low) = split_bytes(value);
            self.set_8bit_register(r2, low);
            self.set_8bit_register(register, high);
        } else if register == Register::SP {
            self.sp += 1;
        } else {
            return Err(EmulateError::UnsupportedRegister {
                opcode: Opcode::INX(register),
                register,
            });
        }
        Ok(())
    }

    pub(crate) fn dcr(
        &mut self,
        register: Register,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        let value = match register {
            Register::SP => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::DCR(register),
                    register,
                })
            }
            Register::M => {
                let (v, _c) = interconnect.read_byte(self.m()).complement_sub(1);
                interconnect.write_byte(self.m(), v);
                v
            }
            _r => {
                let (v, _c) = self.get_8bit_register(_r).unwrap().complement_sub(1);
                self.set_8bit_register(_r, v);
                v
            }
        };
        self.flags.set_non_carry_flags(value);
        Ok(())
    }

    pub(crate) fn add(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        let (result, cy) = match register {
            Register::SP => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::ADD(register),
                    register,
                })
            }
            Register::M => self
                .get_8bit_register(Register::A)?
                .overflowing_add(interconnect.read_byte(self.m())),
            _r => self
                .get_8bit_register(Register::A)?
                .overflowing_add(self.get_8bit_register(_r)?),
        };

        self.flags.set_non_carry_flags(result);
        self.flags.cy = cy;
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(crate) fn adi(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let (result, cy) = self.get_8bit_register(Register::A)?.overflowing_add(value);
            self.flags.set_non_carry_flags(result);
            self.flags.cy = cy;
            self.set_8bit_register(Register::A, result);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::ADI,
                data,
            });
        }
        Ok(())
    }

    pub(crate) fn dad(&mut self, reg: Register) -> Result<()> {
        let addend1 = self.m();
        let addend2 = match (reg, reg.get_pair()) {
            (_r, Some(r2)) => {
                concat_bytes(self.get_8bit_register(_r)?, self.get_8bit_register(r2)?)
            }
            (Register::SP, _) => self.sp,
            (_, _) => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::DAD(reg),
                    register: reg,
                })
            }
        };
        let (result, cy) = addend1.overflowing_add(addend2);
        self.flags.cy = cy;
        self.set_m(result);
        Ok(())
    }

    pub(crate) fn sub(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        let (result, cy) = match register {
            Register::SP => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::SUB(register),
                    register,
                })
            }
            Register::M => self
                .get_8bit_register(Register::A)?
                .complement_sub(interconnect.read_byte(self.m())),
            _r => self
                .get_8bit_register(Register::A)?
                .complement_sub(self.get_8bit_register(_r)?),
        };

        self.flags.set_non_carry_flags(result);
        self.flags.cy = cy;
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(crate) fn sui(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let (result, cy) = self.get_8bit_register(Register::A)?.complement_sub(value);
            self.flags.set_non_carry_flags(result);
            self.flags.cy = cy;
            self.set_8bit_register(Register::A, result);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::SUI,
                data,
            });
        }
        Ok(())
    }

    pub(crate) fn rrc(&mut self) -> Result<()> {
        self.set_8bit_register(Register::A, self.a.rotate_right(1));
        self.flags.cy = self.a & 0x80 != 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::i8080::*;
    use crate::Emulator;
    use std::u8;

    #[test]
    fn overflow_sub() {
        let m: u8 = 0x3e;
        let s: u8 = 0x3e;
        let t = m.complement_sub(s);
        assert_eq!(t, (0, false));

        let m: u8 = 0x00;
        let s: u8 = 0x01;
        assert_eq!(m.complement_sub(s), (u8::MAX, true));
    }

    #[test]
    fn add() {
        let bytecode = [
            0x80, // ADD B
            0x87, // ADD A
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x2e;
        system.cpu.b = 0x6c;
        system.step();
        assert_eq!(system.cpu.a, 0x9a);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, true);

        system.step();
        assert_eq!(system.cpu.a, 0x34);
        assert_eq!(system.cpu.flags.cy, true);
        assert_eq!(system.cpu.flags.p, false);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, false);
    }

    #[test]
    fn adi() {
        let bytecode = [
            0xc6, 0x6c, // ADI 0x6c
            0xc6, 0x9a, // ADI 0x9a
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x2e;
        system.step();
        assert_eq!(system.cpu.a, 0x9a);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, true);

        system.step();
        assert_eq!(system.cpu.a, 0x34);
        assert_eq!(system.cpu.flags.cy, true);
        assert_eq!(system.cpu.flags.p, false);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, false);
    }

    #[test]
    fn sub() {
        let bytecode = [
            0x90, // SUB B
            0x97, // SUB A
        ];
        let mut system = Emulator::new(&bytecode); // SUB B
        system.cpu.a = 0x49;
        system.cpu.b = 0x3a;
        system.step();
        assert_eq!(system.cpu.a, 0x0f);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, false);

        system.cpu.flags.cy = true; //Regression: sub(A) should clear carry bit
        system.step();
        assert_eq!(system.cpu.a, 0x00);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, true);
        assert_eq!(system.cpu.flags.s, false);
    }

    #[test]
    fn sui() {
        let bytecode = [
            0xd6, 0x3a, // SUI 0x3a
            0xd6, 0x0f, // SUI 0x0f
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0x49;
        system.step();
        assert_eq!(system.cpu.a, 0x0f);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, false);
        assert_eq!(system.cpu.flags.s, false);

        system.step();
        assert_eq!(system.cpu.a, 0x00);
        assert_eq!(system.cpu.flags.cy, false);
        assert_eq!(system.cpu.flags.p, true);
        assert_eq!(system.cpu.flags.z, true);
        assert_eq!(system.cpu.flags.s, false);
    }

    #[test]
    fn rrc() {
        let bytecode = [
            0x0f, // RRC
            0x0f, // RRC
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.a = 0xf2;
        system.step();
        assert_eq!(system.cpu.a, 0x79);
        assert_eq!(system.cpu.flags.cy, false);
        system.cpu.a = 0x11;
        system.step();
        assert_eq!(system.cpu.a, 0x88);
        assert_eq!(system.cpu.flags.cy, true);
    }
}
