use crate::{
    cpu8080::{error::EmulateError, Cpu8080, Register},
    instruction::{InstructionData, Opcode},
};

impl<'a> Cpu8080<'a> {
    pub(super) fn out(&mut self, data: InstructionData) -> Result<(), EmulateError> {
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
