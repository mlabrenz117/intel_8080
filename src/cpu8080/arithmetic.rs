use super::*;
use failure::Error;

trait TwosComplement<RHS = Self> {
    type Output;
    fn complement_sub(self, subtrahend: RHS) -> Self::Output;
}

impl TwosComplement for u8 {
    type Output = (u8, bool);
    fn complement_sub(self, subtrahend: Self) -> Self::Output {
        let complement = !subtrahend + 1;
        self.overflowing_add(complement)
    }
}

impl<'a> Cpu8080<'a> {
    pub(super) fn inx(&mut self, register: Register) -> Result<(), Error> {
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
            bail!("INX Does not support register {:?}", register);
        }
        Ok(())
    }

    pub(super) fn dcr(&mut self, register: Register) -> Result<(), Error> {
        match register {
            Register::SP => bail!("DCR does not support SP Register"),
            Register::M => self.set_mem_val(self.get_mem_val() - 1),
            _r => self.set_8bit_register(_r, self.get_8bit_register(_r).unwrap() - 1),
        }
        Ok(())
    }

    pub(super) fn add(&mut self, register: Register) -> Result<(), Error> {
        let (result, cy) = match register {
            Register::SP => bail!("Cannot ADD using SP Register"),
            Register::M => self
                .get_8bit_register(Register::A)?
                .overflowing_add(self.get_mem_loc(self.m())),
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

    pub(super) fn adi(&mut self, value: u8) -> Result<(), Error> {
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
        Ok(())
    }

    pub(super) fn sub(&mut self, register: Register) -> Result<(), Error> {
        let (result, cy) = match register {
            Register::SP => bail!("Cannot SUB using SP Register"),
            Register::M => self
                .get_8bit_register(Register::A)?
                .complement_sub(self.get_mem_loc(self.m())),
            _r => self
                .get_8bit_register(Register::A)?
                .complement_sub(self.get_8bit_register(_r)?),
        };

        self.flags.z = result == 0;
        self.flags.s = result & 0x80 != 0;
        self.flags.cy = !cy;
        self.flags.p = check_parity(result);
        self.set_8bit_register(Register::A, result);
        Ok(())
    }

    pub(super) fn sui(&mut self, value: u8) -> Result<(), Error> {
        let (result, cy) = self.get_8bit_register(Register::A)?.complement_sub(value);
        self.flags.z = result == 0;
        self.flags.s = result & 0x80 != 0;
        self.flags.cy = !cy;
        self.flags.p = check_parity(result);
        self.set_8bit_register(Register::A, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu8080::{Cpu8080, Register};
    #[test]
    fn overflow_sub() {
        let m: u8 = 0x3e;
        let s: u8 = 0x3e;
        let t = m.complement_sub(s);
        assert_eq!(t, (0, true));
    }

    #[test]
    fn add() {
        let mut cpu = Cpu8080::new(&[0x80, 0x87]);
        cpu.set_8bit_register(Register::A, 0x2e);
        cpu.set_8bit_register(Register::B, 0x6c);
        cpu.add(Register::B).unwrap();
        assert_eq!(cpu.a, 0x9a);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);

        cpu.add(Register::A).unwrap();
        assert_eq!(cpu.a, 0x34);
        assert_eq!(cpu.flags.cy, true);
        assert_eq!(cpu.flags.p, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn adi() {
        let mut cpu = Cpu8080::new(&[0xc6, 0x6c, 0xc6, 0x9a]);
        cpu.set_8bit_register(Register::A, 0x2e);
        cpu.adi(0x6c).unwrap();
        assert_eq!(cpu.a, 0x9a);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);

        cpu.adi(0x9a).unwrap();
        assert_eq!(cpu.a, 0x34);
        assert_eq!(cpu.flags.cy, true);
        assert_eq!(cpu.flags.p, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn sub() {
        let mut cpu = Cpu8080::new(&[0x90]);
        cpu.set_8bit_register(Register::A, 0x49);
        cpu.set_8bit_register(Register::B, 0x3a);
        cpu.sub(Register::B).unwrap();
        assert_eq!(cpu.a, 0x0f);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);

        cpu.flags.cy = true; //Regression: sub(A) should clear carry bit
        cpu.sub(Register::A).unwrap();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
    }

    #[test]
    fn sui() {
        let mut cpu = Cpu8080::new(&[0xd6, 0x3a, 0xd6, 0x0f]);
        cpu.set_8bit_register(Register::A, 0x49);
        cpu.sui(0x3a).unwrap();
        assert_eq!(cpu.a, 0x0f);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);

        cpu.sui(0x0f).unwrap();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flags.cy, false);
        assert_eq!(cpu.flags.p, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
    }
}