use super::{Cpu8080, Result};

impl<'a> Cpu8080<'a> {
    pub(super) fn ei(&mut self) -> Result<()> {
        self.interrupts_enabled = true;
        Ok(())
    }
}
