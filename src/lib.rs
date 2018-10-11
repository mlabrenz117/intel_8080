#[macro_use]
extern crate failure;

pub mod cpu8080;
pub mod instruction;

pub use self::cpu8080::Cpu8080;
