use std::{fs::File, io::prelude::*};
#[macro_use]
extern crate failure;

mod cpu8080;
mod disassembler;

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
    cpu.start() //.run_num(1560)
                //let dis = self::disassembler::Disassembler::new(&buf);

    //let mut line: usize = 0x00;
    //for instruction in dis {
    //    println!("{:04x} {}", line, instruction);
    //    line += 1;
    //}
}
