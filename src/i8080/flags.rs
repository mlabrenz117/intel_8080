#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ConditionalFlags {
    pub(crate) z: bool,
    pub(crate) s: bool,
    pub(crate) p: bool,
    pub(crate) cy: bool,
    pub(crate) ac: bool,
}

impl ConditionalFlags {
    pub fn new() -> ConditionalFlags {
        ConditionalFlags {
            z: false,
            s: false,
            p: false,
            cy: false,
            ac: false,
        }
    }

    pub fn check_parity(num: u8) -> bool {
        let mut bytes = num;
        let mut parity = 0;
        while bytes > 0 {
            parity ^= bytes % 2;
            bytes >>= 1;
        }
        parity == 0
    }

    pub fn z(&self) -> bool {
        self.z
    }

    pub fn s(&self) -> bool {
        self.s
    }

    pub fn p(&self) -> bool {
        self.p
    }

    pub fn cy(&self) -> bool {
        self.cy
    }

    pub fn ac(&self) -> bool {
        self.ac
    }

    pub(crate) fn set_non_carry_flags(&mut self, value: u8) {
        self.z = value == 0;
        self.s = value & 0x80 != 0;
        self.p = ConditionalFlags::check_parity(value);
    }
}

impl From<ConditionalFlags> for u8 {
    fn from(flag: ConditionalFlags) -> u8 {
        let s = (flag.s as u8) << 7;
        let z = (flag.z as u8) << 6;
        let ac = (flag.ac as u8) << 4;
        let p = (flag.p as u8) << 2;
        let c = flag.cy as u8;
        s | z | ac | p | c | 2
    }
}

impl From<u8> for ConditionalFlags {
    fn from(byte: u8) -> ConditionalFlags {
        ConditionalFlags {
            s: byte & 0x80 != 0x00,
            z: byte & 0x40 != 0x00,
            ac: byte & 0x10 != 0x00,
            p: byte & 0x04 != 0x00,
            cy: byte & 0x01 != 0x00,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConditionalFlags;
    #[test]
    fn can_test_parity() {
        let odd = 0x5b; // 91
        assert_eq!(ConditionalFlags::check_parity(odd), false);
        let even = 0x9f; // 159
        assert_eq!(ConditionalFlags::check_parity(even), true);
    }

    #[test]
    fn flags_as_bytes() {
        let mut f = ConditionalFlags::new();
        assert_eq!(u8::from(f), 0x02);
        f.s = true;
        assert_eq!(u8::from(f), 0x82);
        f.z = true;
        assert_eq!(u8::from(f), 0xc2);
        f.ac = true;
        assert_eq!(u8::from(f), 0xd2);
        f.p = true;
        assert_eq!(u8::from(f), 0xd6);
        f.cy = true;
        assert_eq!(u8::from(f), 0xd7);
    }
}
