use super::Cpu8080;
use failure::Error;

impl<'a> Cpu8080<'a> {
    pub(super) fn jmp(&mut self, addr: u16) -> Result<(), Error> {
        self.disassembler.update_pc(addr);
        Ok(())
    }

    pub(super) fn call(&mut self, addr: u16) -> Result<(), Error> {
        self.push_u16(self.disassembler.pc())?;
        self.disassembler.update_pc(addr);
        Ok(())
    }
}
