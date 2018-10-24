use crate::i8080::{Result, I8080};

impl I8080 {
    pub(crate) fn ei(&mut self) -> Result<()> {
        self.interrupts_enabled = true;
        Ok(())
    }
}
