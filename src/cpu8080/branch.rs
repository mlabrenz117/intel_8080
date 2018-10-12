use super::{concat_bytes, error::EmulateError, Cpu8080};
use crate::instruction::{InstructionData, Opcode};

impl<'a> Cpu8080<'a> {
    pub(super) fn jmp(&mut self, data: InstructionData) -> Result<(), EmulateError> {
        if let (Some(hi), Some(lo)) = data.tuple() {
            let addr = concat_bytes(hi, lo);
            self.pc.addr = addr;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::JMP,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn jnz(&mut self, data: InstructionData) -> Result<(), EmulateError> {
        if !self.flags.z {
            self.jmp(data)?;
        }
        Ok(())
    }

    pub(super) fn call(&mut self, data: InstructionData) -> Result<(), EmulateError> {
        if let (Some(hi), Some(lo)) = data.tuple() {
            let addr = concat_bytes(hi, lo);
            self.push_u16(self.pc.addr)?;
            self.pc.addr = addr;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::CALL,
                data,
            });
        }
        Ok(())
    }

    pub(super) fn ret(&mut self) -> Result<(), EmulateError> {
        let addr = self.pop_u16()?;
        self.pc.addr = addr;
        Ok(())
    }
}
