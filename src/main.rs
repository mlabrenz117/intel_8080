use std::{fs::File, io::prelude::*};
#[macro_use]
extern crate failure;

pub mod cpu8080;
pub mod instruction;

use self::cpu8080::Cpu8080;

fn main() -> Result<(), failure::Error> {
    let mut f = File::open("resources/invaders.rom")?;
    let mut str_buf = String::new();
    let mut buf = [0 as u8; 0x2000];

    f.read_to_string(&mut str_buf)?;

    let mut idx = 0;
    let lines = str_buf.lines();
    for line in lines {
        let mut line = line.to_string();
        line.pop();
        for byte in line.split(',') {
            if byte.len() == 4 {
                let b: u8 = u8::from_str_radix(&byte[2..], 16).unwrap();
                buf[idx] = b;
                idx += 1;
            }
        }
    }

    let mut cpu = Cpu8080::new(&buf);
    cpu.start()?;

    Ok(())
}
