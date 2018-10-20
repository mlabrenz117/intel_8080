use crate::{
    cpu8080::{error::EmulateError, Cpu8080, Register, Result},
    instruction::{InstructionData, Opcode},
};

impl<'a> Cpu8080<'a> {
    pub(super) fn out(&mut self, data: InstructionData) -> Result<()> {
        if let Some(device) = data.first() {
            self.write_device(device, self.get_8bit_register(Register::A)?);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::OUT,
                data,
            });
        }
        Ok(())
    }
}
