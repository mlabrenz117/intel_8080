use crate::i8080::concat_bytes;
use std::fmt::{self, Display};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InstructionData {
    first: Option<u8>,
    second: Option<u8>,
}

impl InstructionData {
    pub fn new(first: Option<u8>, second: Option<u8>) -> Self {
        match (first, second) {
            (None, Some(s)) => InstructionData {
                first: Some(s),
                second: None,
            },
            (_, _) => InstructionData { first, second },
        }
    }

    pub fn first(&self) -> Option<u8> {
        self.first
    }

    pub fn addr(&self) -> Option<u16> {
        if let (Some(h), Some(l)) = self.tuple() {
            return Some(concat_bytes(h, l));
        }
        None
    }

    pub fn tuple(&self) -> (Option<u8>, Option<u8>) {
        (self.first, self.second)
    }
}

impl Display for InstructionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.tuple() {
            (Some(hi), Some(lo)) => {
                let value = concat_bytes(hi, lo);
                write!(f, "0x{:04x}", value)
            }
            (Some(byte), None) => write!(f, "0x{:02x}", byte),
            (_, _) => write!(f, ""),
        }
    }
}
