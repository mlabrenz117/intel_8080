use crate::mem_map::{WRAM_END, WRAM_START};

pub struct Wram {
    bytes: Box<[u8]>,
}

impl Wram {
    pub fn new() -> Wram {
        const LENGTH: usize = (WRAM_END - WRAM_START + 1) as usize;
        Wram {
            bytes: Box::new([0; LENGTH]),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.bytes[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.bytes[addr as usize] = value;
    }
}
