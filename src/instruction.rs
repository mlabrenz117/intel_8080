pub mod opcode;
pub use self::opcode::Opcode;

mod instruction_data;
pub(crate) use self::instruction_data::InstructionData;

use crate::i8080::split_bytes;
use failure::bail;
use failure::Error;
use std::fmt::{self, Display};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    opcode: Opcode,
    data: InstructionData,
}

impl Instruction {
    pub fn new_unary(opcode: Opcode) -> Result<Instruction, Error> {
        if let self::opcode::OpcodeSize::Unary = opcode.size() {
            Ok(Instruction {
                opcode,
                data: InstructionData::new(None, None),
                //params: InstructionParams::Unary,
            })
        } else {
            bail!("Unary instructions require Unary Opcodes")
        }
    }

    pub fn new_binary(opcode: Opcode, data: u8) -> Result<Instruction, Error> {
        if let self::opcode::OpcodeSize::Binary = opcode.size() {
            Ok(Instruction {
                opcode,
                data: InstructionData::new(Some(data), None),
                //params: InstructionParams::Binary(data),
            })
        } else {
            bail!("Binary instructions require Binary Opcodes")
        }
    }

    pub fn new_trinary(opcode: Opcode, addr: u16) -> Result<Instruction, Error> {
        if let self::opcode::OpcodeSize::Trinary = opcode.size() {
            let (h, l) = split_bytes(addr);
            Ok(Instruction {
                opcode,
                data: InstructionData::new(Some(h), Some(l)),
                //params: InstructionParams::Trinary(addr),
            })
        } else {
            bail!("Trinary instructions require Trinary Opcodes")
        }
    }

    pub fn len(&self) -> u16 {
        match self.opcode.size() {
            self::opcode::OpcodeSize::Unary => 1,
            self::opcode::OpcodeSize::Binary => 2,
            self::opcode::OpcodeSize::Trinary => 3,
        }
    }

    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    pub fn data(&self) -> InstructionData {
        self.data
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.len(), self.opcode.num_registers()) {
            (1, 0) => write!(f, "{}           ", self.opcode),
            (1, 1) => write!(f, "{}          ", self.opcode),
            (1, 2) => write!(f, "{}        ", self.opcode),
            (2, 0) => write!(f, "{}{}       ", self.opcode, self.data),
            (2, 1) => write!(f, "{}, {}    ", self.opcode, self.data),
            (3, 0) => write!(f, "{}{}     ", self.opcode, self.data),
            (3, 1) => write!(f, "{}, {}  ", self.opcode, self.data),
            (_, _) => write!(f, ""),
        }
    }
}
