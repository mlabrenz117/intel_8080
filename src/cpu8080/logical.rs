use super::{check_parity, Cpu8080, TwosComplement};
use crate::cpu8080::error::EmulateError;
use crate::instruction::{InstructionData, Opcode};

impl<'a> Cpu8080<'a> {
    pub(super) fn cpi(&mut self, data: InstructionData) -> Result<(), EmulateError> {
        if let Some(value) = data.first() {
            let (v, c) = self.a.complement_sub(value);
            self.flags.z = v == 0;
            self.flags.s = v & 0x80 != 0;
            self.flags.p = check_parity(v);
            self.flags.cy = c;
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::CPI,
                data,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu8080::Cpu8080;
    #[test]
    fn cpi() {
        let bytecode = [
            0xfe, 0x6f, // CPI 0x6f
            0xfe, 0x5f, // CPI 0x5f
            0xfe, 0x4f, // CPI 0x4f
        ];
        let mut cpu = Cpu8080::new(&bytecode);
        cpu.a = 0x5f;
        cpu.step().unwrap();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);
        assert_eq!(cpu.flags.cy, true);
        cpu.step().unwrap();
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
        cpu.step().unwrap();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
    }
}
