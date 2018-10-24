use crate::{
    i8080::{error::EmulateError, Result, I8080},
    instruction::{InstructionData, Opcode},
};
use log::warn;

impl I8080 {
    pub(crate) fn out(&mut self, data: InstructionData) -> Result<()> {
        if let Some(_device) = data.first() {
            // TODO: self.write_device(device, self.get_8bit_register(Register::A)?);
            warn!("Devices unimplemented");
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::OUT,
                data,
            });
        }
        Ok(())
    }
}
