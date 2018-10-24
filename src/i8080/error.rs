use crate::{
    i8080::Register,
    instruction::{Instruction, InstructionData, Opcode},
};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum EmulateError {
    #[fail(
        display = "{:?} is unsupported for Opcode {}",
        register,
        opcode
    )]
    UnsupportedRegister { opcode: Opcode, register: Register },
    #[fail(
        display = "bad instruction data: {} for opcode: {}",
        data,
        opcode
    )]
    InvalidInstructionData {
        data: InstructionData,
        opcode: Opcode,
    },
    #[fail(display = "Instruciton not yet implemented: {}", instruction)]
    UnimplementedInstruction { instruction: Instruction },
    #[fail(display = "{:?} is not an 8 bit register", register)]
    RegisterNot8Bit { register: Register },
    #[fail(display = "Stack Overflow")]
    StackOverflow,
    #[fail(display = "Tring to write to ROM")]
    WriteToROM,
}
