use log::error;

//use crate::game_pad::GamePad;
use crate::mem_map::*;
use crate::rom::Rom;
use crate::vram::Vram;
use crate::wram::Wram;

pub struct Interconnect {
    rom: Rom,
    wram: Wram,
    vram: Vram,
    //game_pad: GamePad,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect {
            rom,
            wram: Wram::new(),
            vram: Vram::new(),
            //game_pad: GamePad::new(),
        }
    }

    pub fn rom_len(&self) -> usize {
        self.rom.len()
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            ROM_START...ROM_END => self.rom.read_byte(addr - ROM_START),
            WRAM_START...WRAM_END => self.wram.read_byte(addr - WRAM_START),
            VRAM_START...VRAM_END => self.vram.read_byte(addr - VRAM_START),
            _ => panic!("Unrecognized Address: 0x{:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            ROM_START...ROM_END => error!("Attempting to write to ROM"),
            WRAM_START...WRAM_END => self.wram.write_byte(addr - WRAM_START, value),
            VRAM_START...VRAM_END => self.vram.write_byte(addr - VRAM_START, value),
            _ => panic!("Unrecognized Address: 0x{:04x}", addr),
        }
    }
}
