use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    SP,
}

impl Register {
    pub fn get_pair(&self) -> Option<Register> {
        match self {
            Register::B => Some(Register::C),
            Register::D => Some(Register::E),
            Register::H => Some(Register::L),
            _ => None,
        }
    }

    pub fn is_8bit(&self) -> bool {
        match self {
            Register::A => true,
            Register::B => true,
            Register::C => true,
            Register::D => true,
            Register::E => true,
            Register::H => true,
            Register::L => true,
            Register::M => false,
            Register::SP => false,
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Register::A => "A",
            Register::B => "B",
            Register::C => "C",
            Register::D => "D",
            Register::E => "E",
            Register::H => "H",
            Register::L => "L",
            Register::M => "M",
            Register::SP => "S",
        };
        write!(f, "{}", s)
    }
}
