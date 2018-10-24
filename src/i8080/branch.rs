use super::{concat_bytes, error::EmulateError, Result, I8080};
use crate::instruction::{InstructionData, Opcode};
use crate::interconnect::Interconnect;

impl I8080 {
    pub(super) fn jmp(&mut self, data: InstructionData) -> Result<()> {
        if let (Some(hi), Some(lo)) = data.tuple() {
            let addr = concat_bytes(hi, lo);
            self.pc = addr;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::JMP,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn jnz(&mut self, data: InstructionData) -> Result<()> {
        if !self.flags.z {
            self.jmp(data)?;
        }
        Ok(())
    }

    pub(super) fn call(
        &mut self,
        data: InstructionData,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        if let (Some(hi), Some(lo)) = data.tuple() {
            let addr = concat_bytes(hi, lo);
            self.push_u16(self.pc, interconnect)?;
            self.pc = addr;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::CALL,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn ret(&mut self, interconnect: &mut Interconnect) -> Result<()> {
        let addr = self.pop_u16(interconnect)?;
        self.pc = addr;
        Ok(())
    }
}
