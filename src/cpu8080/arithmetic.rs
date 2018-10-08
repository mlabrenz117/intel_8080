use super::{check_parity, ConditionalFlags, Cpu8080, Register};
use failure::Error;

impl<'a> Cpu8080<'a> {
    pub(super) fn add(&mut self, register: Register) -> Result<(), Error> {
        if register == Register::SP {}

        let result = match register {
            Register::SP => bail!("Cannot ADD using SP Register"),
            Register::M => {
                self.get_8bit_register(Register::A)? as u16
                    + self.get_mem_loc(self.get_m_register()) as u16
            }
            _r => self.get_8bit_register(Register::A)? as u16 + self.get_8bit_register(_r)? as u16,
        };

        // Set Zero Flag
        self.flags.z = result & 0xff == 0;

        // Set Sign flag (if bit 7 is set)
        self.flags.s = result & 0x80 != 0;

        //Set Carry Flag
        self.flags.cy = result > 0xff;

        //Set Parity Flag
        self.flags.p = check_parity(result & 0xff);

        //Update the register
        self.set_8bit_register(Register::A, (result & 0xff) as u8);
        Ok(())
    }

    pub(super) fn adi(&mut self, value: u8) -> Result<(), Error> {
        let result: u16 = self.get_8bit_register(Register::A)? as u16 + value as u16;
        self.flags.z = result & 0xff == 0;

        // Set Sign flag (if bit 7 is set)
        self.flags.s = result & 0x80 != 0;

        //Set Carry Flag
        self.flags.cy = result > 0xff;

        //Set Parity Flag
        self.flags.p = check_parity(result & 0xff);

        //Update the register
        self.set_8bit_register(Register::A, (result & 0xff) as u8);
        Ok(())
    }
}
