use super::Cpu8080;
use failure::Error;

impl<'a> Cpu8080<'a> {
    pub(super) fn jmp(&mut self, addr: u16) -> Result<(), Error> {
        self.pc = addr;
        Ok(())
    }

    pub(super) fn jnz(&mut self, addr: u16) -> Result<(), Error> {
        if !self.flags.z {
            self.jmp(addr)?;
        }
        Ok(())
    }

    pub(super) fn call(&mut self, addr: u16) -> Result<(), Error> {
        self.push_u16(self.pc)?;
        self.pc = addr;
        Ok(())
    }

    pub(super) fn ret(&mut self) -> Result<(), Error> {
        let addr = self.pop_u16()?;
        self.pc = addr;
        Ok(())
    }
}
