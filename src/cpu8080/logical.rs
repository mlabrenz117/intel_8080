use super::{check_parity, Cpu8080, TwosComplement};
use failure::Error;

impl<'a> Cpu8080<'a> {
    pub(super) fn cpi(&mut self, data: u8) -> Result<(), Error> {
        let (v, c) = self.a.complement_sub(data);
        self.flags.z = v == 0;
        self.flags.s = v & 0x80 != 0;
        self.flags.p = check_parity(v);
        self.flags.cy = c;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu8080::Cpu8080;
    #[test]
    fn cpi() {
        let mut cpu = Cpu8080::new(&[]);
        cpu.a = 0x5f;
        cpu.cpi(0x6f).unwrap();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, true);
        assert_eq!(cpu.flags.cy, true);
        cpu.cpi(0x5f).unwrap();
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
        cpu.cpi(0x4f).unwrap();
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.s, false);
        assert_eq!(cpu.flags.cy, false);
    }
}
