use super::{split_bytes, Cpu8080, Register};
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

    pub(super) fn mov(&mut self, destination: Register, source: Register) -> Result<(), Error> {
        if destination == Register::SP || source == Register::SP {
            bail!("Cannot move using SP register")
        };
        if destination != Register::M && source != Register::M {
            self.set_8bit_register(destination, self.get_8bit_register(source)?);
        } else {
            unimplemented!("Move the M register into 8 bit registers?")
        }
        Ok(())
    }
}
