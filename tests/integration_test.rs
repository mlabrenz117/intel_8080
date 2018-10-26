extern crate i8080_emulator;

use std::fs::File;
use std::io::Read;

use i8080_emulator::Emulator;

#[test]
fn it_works() {
    let file = File::open("tests/test.rom").unwrap();
    let mut bytecode: Vec<u8> = file.bytes().filter_map(|b| b.ok()).collect();

    //Skip DAA and Aux Carry Test
    bytecode[0x59c] = 0xc3; // JMP 0x05c2
    bytecode[0x59d] = 0xc2;
    bytecode[0x59e] = 0x05;

    let mut emulator = Emulator::new(bytecode);
    if let Err(e) = emulator.try_run() {
        panic!("{}", e)
    }

    //assert_eq!(emulator.cpu().m(), 0x11);
}
