use crate::mem_map::{VRAM_END, VRAM_START};

pub struct Vram {
    bytes: Box<[u8]>,
}

impl Vram {
    pub fn new() -> Vram {
        const LENGTH: usize = (VRAM_END - VRAM_START + 1) as usize;
        Vram {
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
