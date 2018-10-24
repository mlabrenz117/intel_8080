use super::{Result, I8080};

impl I8080 {
    pub(super) fn ei(&mut self) -> Result<()> {
        self.interrupts_enabled = true;
        Ok(())
    }
}
