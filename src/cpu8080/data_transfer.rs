use super::{concat_bytes, split_bytes, Cpu8080, Register};
use failure::Error;

impl<'a> Cpu8080<'a> {
    pub(super) fn lxi(&mut self, register: Register, params: u16) -> Result<(), Error> {
        if let Some(r2) = register.get_pair() {
            let (high, low) = split_bytes(params);
            self.set_8bit_register(register, high);
            self.set_8bit_register(r2, low);
        } else if register == Register::SP {
            self.set_sp_register(params);
        } else {
            bail!("LXI does not support register {:?}", register);
        }
        Ok(())
    }

    pub(super) fn ldax(&mut self, register: Register) -> Result<(), Error> {
        let pair = match register {
            Register::B | Register::D => register.get_pair().unwrap(),
            _r => bail!("LDAX does not support register {:?}", register),
        };
        let loc = concat_bytes(
            self.get_8bit_register(register)?,
            self.get_8bit_register(pair)?,
        );
        let value = self.get_mem_loc(loc);
        self.set_8bit_register(Register::A, value);
        Ok(())
    }

    pub(super) fn mov(&mut self, destination: Register, source: Register) -> Result<(), Error> {
        match (destination, source) {
            (Register::SP, _) | (_, Register::SP) => bail!("Cannot move using SP Register"),
            (Register::M, _r) => {
                let addr = concat_bytes(
                    self.get_8bit_register(Register::H).unwrap(),
                    self.get_8bit_register(Register::L).unwrap(),
                );
                self.set_mem_loc(addr, self.get_8bit_register(_r)?);
            }
            (_r, Register::M) => {
                let addr = concat_bytes(
                    self.get_8bit_register(Register::H).unwrap(),
                    self.get_8bit_register(Register::L).unwrap(),
                );
                self.set_8bit_register(_r, self.get_mem_loc(addr));
            }
            (_r1, _r2) => self.set_8bit_register(_r1, self.get_8bit_register(_r2)?),
        }
        Ok(())
    }

    pub(super) fn mvi(&mut self, register: Register, value: u8) -> Result<(), Error> {
        if register == Register::SP || register == Register::A {
            bail!("MVI cannot be used with SP or A Registers");
        };
        self.set_8bit_register(register, value);
        Ok(())
    }
}
