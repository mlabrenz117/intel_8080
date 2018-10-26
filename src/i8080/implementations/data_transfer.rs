use crate::{
    i8080::error::EmulateError,
    i8080::{concat_bytes, Register, Result, I8080},
    instruction::{InstructionData, Opcode},
    interconnect::Interconnect,
};

impl I8080 {
    /// #LXI - Load Register Pair Immediate
    ///
    /// Opcodes: 0x01, 0x11, 0x21, 0x31
    /// Supported Registers: B(0x01), D(0x11), H(0x21), SP(0x31)
    /// Params: Two Bytes following opcode
    ///
    /// Loads the 2 bytes following the opcode into the register
    /// pair determined from the opcode.
    ///
    /// Returns Ok(()) on success.
    ///
    /// #Errors
    /// Fails if given registers A, C, E, L, or M.
    pub(crate) fn lxi(&mut self, register: Register, data: InstructionData) -> Result<()> {
        if let (Some(high), Some(low)) = data.tuple() {
            if let Some(r2) = register.get_pair() {
                self.set_8bit_register(register, high);
                self.set_8bit_register(r2, low);
            } else if register == Register::SP {
                self.set_sp(concat_bytes(high, low));
            } else {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::LXI(register),
                    register,
                });
            }
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::LXI(register),
                data,
            });
        }
        Ok(())
    }

    /// #LDA - Load Accumulator from Memory
    ///
    /// Opcodes: 0x3a,
    /// Params: Two byte memory location following the opcode
    ///
    /// Loads the byte at the memory location given into the accumulator.
    // TODO: WRITE TEST
    pub(crate) fn lda(&mut self, data: InstructionData, interconnect: &Interconnect) -> Result<()> {
        if let Some(addr) = data.addr() {
            self.set_8bit_register(Register::A, interconnect.read_byte(addr));
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::LDA,
                data,
            });
        }
        Ok(())
    }

    /// #STA - Store Accumulator in Memory
    ///
    /// Opcodes: 0x32
    /// Params: Two byte memory location following the opcode
    ///
    /// Stores the value in the accumulator into memory at the given address.
    // TODO: WRITE TEST
    pub(crate) fn sta(
        &mut self,
        data: InstructionData,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        if let Some(addr) = data.addr() {
            interconnect.write_byte(addr, self.a);
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::STA,
                data,
            });
        }
        Ok(())
    }

    /// #LDAX - Load Accumulator
    ///
    /// Opcodes: 0x0a, 0x1a
    /// Supported Registers: B(0x0a), D(0x1a)
    ///
    /// The contents of the memory location addressed by registers BC or DE, replace the contents
    /// of the accumulator.
    ///
    /// #Errors
    /// Fails if given registers A, C, E, H, L, M, SP.
    pub(crate) fn ldax(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        let pair = match register {
            Register::B | Register::D => register.get_pair().unwrap(),
            _r => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::LDAX(register),
                    register,
                })
            }
        };
        let loc = concat_bytes(
            self.get_8bit_register(register)?,
            self.get_8bit_register(pair)?,
        );
        let value = interconnect.read_byte(loc);
        self.set_8bit_register(Register::A, value);
        Ok(())
    }

    /// #MOV - Move
    ///
    /// Opcodes: 0x40 - 0x7f; excluding 0x76
    /// Supported Registers: A, B, C, D, E, H, L, M
    ///
    /// One byte of data is moved from the register specified
    /// by src (the source register) to the register specified
    /// by dst (the destination register). The data replaces
    /// the contents of the destination register; the source remains unchanged.
    ///
    /// NOTE: MOV using the M register moves data out-of or into memory at location (HL).
    ///
    /// #Errors
    /// Fails if given register SP.
    pub(crate) fn mov(
        &mut self,
        destination: Register,
        source: Register,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        match (destination, source) {
            (Register::SP, _) | (_, Register::SP) => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::MOV(destination, source),
                    register: Register::SP,
                })
            }
            (Register::M, _r) => {
                let addr = self.m();
                if _r == Register::M {
                    return Ok(());
                };
                interconnect.write_byte(addr, self.get_8bit_register(_r)?);
            }
            (_r, Register::M) => {
                let addr = self.m();
                self.set_8bit_register(_r, interconnect.read_byte(addr));
            }
            (_r1, _r2) => self.set_8bit_register(_r1, self.get_8bit_register(_r2)?),
        }
        Ok(())
    }

    /// #MVI - Move Immediate Data
    ///
    /// Opcodes: 0x06, 0x0e, 0x16, 0x1e, 0x26, 0x2e, 0x36, 0x3e
    /// Supported Registers: A(0x3e), B(0x06), C(0x0e), D(0x16),
    ///                      E(0x1e), H(0x26), L(0x2e), M(0x36)
    ///
    /// The byte of immediate data is stored in the specified register or memory byte.
    ///
    /// #Errors
    /// Fails if given register SP.
    pub(crate) fn mvi(
        &mut self,
        register: Register,
        data: InstructionData,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        if let (Some(value), None) = data.tuple() {
            match register {
                Register::SP => {
                    return Err(EmulateError::UnsupportedRegister {
                        opcode: Opcode::MVI(register),
                        register,
                    })
                }
                Register::M => {
                    interconnect.write_byte(self.m(), value);
                }
                _r => {
                    self.set_8bit_register(register, value);
                }
            }
        } else {
            return Err(EmulateError::InvalidInstructionData {
                opcode: Opcode::MVI(register),
                data,
            });
        }
        Ok(())
    }

    ///PUSH - Push Data Onto Stack
    ///
    /// Opcodes: 0xc5, 0xd5, 0xe5, 0xf5
    /// Supported Registers: B(0xc5), D(0xd5), H(0xe5), SP(0xf5)
    /// NOTE: We use Register::A to indicate PSW
    ///
    /// The contest of the specified register pair are saved in
    /// two bytes of memory indicated by the stack pointer SP.
    ///
    /// The contents of the first register are saved at memory
    /// address one less than the address indicated by the stack pointer.
    /// If register A(PSW) is specified the first bye of information saved holds the contents of the A
    /// register; the second byte holds the settings of the five condition flags (Carry, Zero,
    /// Sign, Parity, and Aux Carry.
    ///
    /// #Errors
    /// Fails if given registers A, C, E, L, or M
    pub(crate) fn push(
        &mut self,
        register: Register,
        interconnect: &mut Interconnect,
    ) -> Result<()> {
        match (register, register.get_pair()) {
            (_r, Some(r2)) => {
                let value = concat_bytes(self.get_8bit_register(_r)?, self.get_8bit_register(r2)?);
                self.push_u16(value, interconnect)?;
            }
            (Register::A, None) => {
                let value =
                    concat_bytes(self.get_8bit_register(Register::A)?, u8::from(self.flags));
                self.push_u16(value, interconnect)?;
            }
            (_r, _) => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::PUSH(register),
                    register,
                })
            }
        };
        Ok(())
    }

    /// Pop - Pop Data Off Stack
    ///
    /// Opcodes: 0xc1, 0xd1, 0xe1, 0xf1
    /// Supported Registers: B(0xc1), D(0xd1), H(0xe1), SP(0xf1)
    /// NOTE: We use Register::A to indicate PSW
    ///
    /// The contents of the specified register pair are restored from two bytes of memory indicated
    /// by the Stack Pointer, SP. The byte of memory indicated by the stack pointer is loaded into
    /// the second register of the register pair. The byte of of memory at the address one greater
    /// than the stack pointer is loaded into the first register of the register pair, unless PSW
    /// is indicated, then it is loaded into the conditional flags.
    ///
    /// The Stack Pointer is incremented by 2.
    pub(crate) fn pop(&mut self, register: Register, interconnect: &Interconnect) -> Result<()> {
        use crate::i8080::flags::ConditionalFlags;
        match (register, register.get_pair()) {
            (_r, Some(r2)) => {
                let low = self.pop_u8(interconnect)?;
                let high = self.pop_u8(interconnect)?;
                self.set_8bit_register(_r, high);
                self.set_8bit_register(r2, low);
            }
            (Register::A, None) => {
                let flags = self.pop_u8(interconnect)?;
                let a = self.pop_u8(interconnect)?;
                self.flags = ConditionalFlags::from(flags);
                self.set_8bit_register(Register::A, a);
            }
            (_r, _) => {
                return Err(EmulateError::UnsupportedRegister {
                    opcode: Opcode::POP(register),
                    register,
                })
            }
        }
        Ok(())
    }

    ///XCHG - Exchange Registers
    ///
    /// Opcodes: 0xeb
    /// Unary Opcode, No register
    ///
    /// The 16 bits of data held in the H and L registers are exchanged with the 16 bits of data
    /// held in the D and E registers.
    ///
    /// Condition flags affected: None,
    pub(crate) fn xchg(&mut self) -> Result<()> {
        let l = self.l;
        let h = self.h;
        self.set_8bit_register(Register::L, self.e);
        self.set_8bit_register(Register::H, self.d);
        self.set_8bit_register(Register::D, h);
        self.set_8bit_register(Register::E, l);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::interconnect::Rom;
    use crate::Emulator;

    #[test]
    fn lxi() {
        let bytecode = [
            0x01, 0xcc, 0xbb, //LXI B, 0xbbcc
            0x11, 0xee, 0xdd, //LXI D, 0xddee
            0x21, 0x11, 0xff, //LXI H, 0xff11
            0x31, 0xbb, 0xaa, //LXI SP, 0xaabb
        ];
        let mut system = Emulator::new(Rom::from(&bytecode));
        system.run();
        assert_eq!(system.cpu.b, 0xbb);
        assert_eq!(system.cpu.c, 0xcc);
        assert_eq!(system.cpu.d, 0xdd);
        assert_eq!(system.cpu.e, 0xee);
        assert_eq!(system.cpu.h, 0xff);
        assert_eq!(system.cpu.l, 0x11);
        assert_eq!(system.cpu.sp, 0xaabb);
    }

    #[test]
    fn ldax() {
        let bytecode = [
            0x0a, // LDAX B
            0x1a, // LDAX D
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.b = 0x20;
        system.cpu.d = 0x20;
        system.cpu.e = 0x01;
        system.interconnect.write_byte(0x2000, 0xaa);
        system.interconnect.write_byte(0x2001, 0xbb);
        system.step();
        assert_eq!(system.cpu.a, 0xaa);
        system.step();
        assert_eq!(system.cpu.a, 0xbb);
    }

    #[test]
    fn mov() {
        let bytecode = [
            0x42, // MOV(B,D)
            0x4e, // MOV(C,M)
            0x77, // MOV(M,A)
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.d = 0xbd;
        system.cpu.a = 0xaa;
        system.cpu.h = 0x20;
        system.interconnect.write_byte(0x2000, 0xcc);
        system.step();
        assert_eq!(system.cpu.b, 0xbd);
        system.step();
        assert_eq!(system.cpu.c, 0xcc);
        system.step();
        assert_eq!(system.interconnect.read_byte(0x2000), 0xaa);
    }

    #[test]
    fn mvi() {
        let bytecode = [
            0x26, 0x20, //MVI H, 0x20
            0x36, 0xff, //MVI M, 0xff
        ];
        let mut system = Emulator::new(&bytecode);
        system.run();
        assert_eq!(system.cpu.h, 0x20);
        assert_eq!(system.interconnect.read_byte(0x2000), 0xff);
    }

    #[test]
    fn push() {
        let bytecode = [
            0xd5, // PUSH D
            0xf5, // PUSH PSW
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.sp = 0x2400;
        system.cpu.d = 0x8f;
        system.cpu.e = 0x9d;
        system.cpu.a = 0x1f;
        system.cpu.flags.cy = true;
        system.cpu.flags.z = true;
        system.cpu.flags.p = true;
        system.run();
        assert_eq!(system.interconnect.read_byte(0x2400 - 1), 0x8f);
        assert_eq!(system.interconnect.read_byte(0x2400 - 2), 0x9d);
        assert_eq!(system.interconnect.read_byte(0x2400 - 3), 0x1f);
        assert_eq!(system.interconnect.read_byte(0x2400 - 4), 0x47);
        assert_eq!(system.cpu.sp, 0x2400 - 4);
    }

    #[test]
    fn pop() {
        use crate::i8080::ConditionalFlags;
        let bytecode = [
            0xf5, // PUSH PSW
            0xc5, // PUSH B
            0xd1, // POP D
            0xf1, // POP PSW
        ];
        let mut system = Emulator::new(&bytecode);
        system.cpu.sp = 0x2400;
        system.cpu.a = 0xaa;
        system.cpu.b = 0xbb;
        system.cpu.flags.cy = true;
        system.cpu.flags.p = true;
        system.step();
        system.step();
        system.cpu.flags = ConditionalFlags::new();
        system.step();
        assert_eq!(system.cpu.d, 0xbb);
        system.step();
        assert_eq!(
            system.cpu.flags,
            ConditionalFlags {
                z: false,
                s: false,
                ac: false,
                p: true,
                cy: true
            }
        );
        assert_eq!(system.cpu.sp, 0x2400);
    }

    #[test]
    fn xchg() {
        let bytecode = [0xeb];
        let mut system = Emulator::new(&bytecode);
        system.cpu.h = 0x00;
        system.cpu.l = 0xff;
        system.cpu.d = 0x33;
        system.cpu.e = 0x55;
        system.run();
        assert_eq!(system.cpu.h, 0x33);
        assert_eq!(system.cpu.l, 0x55);
        assert_eq!(system.cpu.d, 0x00);
        assert_eq!(system.cpu.e, 0xff);
    }
}
