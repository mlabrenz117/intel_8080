use super::*;
use crate::instruction::{InstructionData, Opcode};
use crate::interconnect::Interconnect;

impl I8080 {
    pub(super) fn inx(&mut self, register: Register) -> Result<()> {
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

    pub(super) fn dcr(
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
        self.flags.z = value == 0;
        self.flags.s = value & 0x80 != 0;
        self.flags.p = check_parity(value);
        Ok(())
    }

    pub(super) fn add(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
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

        // Set Zero Flag
        self.flags.z = result == 0;

        // Set Sign flag (if bit 7 is set)
        self.flags.s = result & 0x80 != 0;

        //Set Carry Flag
        self.flags.cy = cy;

        //Set Parity Flag
        self.flags.p = check_parity(result);

        //Update the register
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(super) fn adi(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let (result, cy) = self.get_8bit_register(Register::A)?.overflowing_add(value);
            self.flags.z = result == 0;

            // Set Sign flag (if bit 7 is set)
            self.flags.s = result & 0x80 != 0;

            //Set Carry Flag
            self.flags.cy = cy;

            //Set Parity Flag
            self.flags.p = check_parity(result);

            //Update the register
            self.set_8bit_register(Register::A, result);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::ADI,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn dad(&mut self, reg: Register) -> Result<()> {
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

    pub(super) fn sub(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
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

        self.flags.z = result == 0;
        self.flags.s = result & 0x80 != 0;
        self.flags.cy = cy;
        self.flags.p = check_parity(result);
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(super) fn sui(&mut self, data: InstructionData) -> Result<()> {
        if let Some(value) = data.first() {
            let (result, cy) = self.get_8bit_register(Register::A)?.complement_sub(value);
            self.flags.z = result == 0;
            self.flags.s = result & 0x80 != 0;
            self.flags.cy = cy;
            self.flags.p = check_parity(result);
            self.set_8bit_register(Register::A, result);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::SUI,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn rrc(&mut self) -> Result<()> {
        self.set_8bit_register(Register::A, self.a.rotate_right(1));
        self.flags.cy = self.a & 0x80 != 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu8080::Cpu8080;
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
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x2e;
        cpu.b = 0x6c;
        cpu.step();
        assert_eq!(cpu.a, 0x9a);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);

        cpu.step();
        assert_eq!(cpu.a, 0x34);
        assert_eq!(cpu.flags.cy, true);
        assert_eq!(cpu.flags.p, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn adi() {
        let bytecode = [
            0xc6, 0x6c, // ADI 0x6c
            0xc6, 0x9a, // ADI 0x9a
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x2e;
        cpu.step();
        assert_eq!(cpu.a, 0x9a);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);

        cpu.step();
        assert_eq!(cpu.a, 0x34);
        assert_eq!(cpu.flags.cy, true);
        assert_eq!(cpu.flags.p, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn sub() {
        let bytecode = [
            0x90, // SUB B
            0x97, // SUB A
        ];
        let mut cpu = Cpu8080::new(&bytecode); // SUB B
        cpu.a = 0x49;
        cpu.b = 0x3a;
        cpu.step();
        assert_eq!(cpu.a, 0x0f);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);

        cpu.flags.cy = true; //Regression: sub(A) should clear carry bit
        cpu.step();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn sui() {
        let bytecode = [
            0xd6, 0x3a, // SUI 0x3a
            0xd6, 0x0f, // SUI 0x0f
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x49;
        cpu.step();
        assert_eq!(cpu.a, 0x0f);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);

        cpu.step();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn rrc() {
        let bytecode = [
            0x0f, // RRC
            0x0f, // RRC
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0xf2;
        cpu.step();
        assert_eq!(cpu.a, 0x79);
        assert_eq!(cpu.flags.cy, false);
        cpu.a = 0x11;
        cpu.step();
        assert_eq!(cpu.a, 0x88);
        assert_eq!(cpu.flags.cy, true);
    }
}
