mod instructions;

use crate::{
    zmachine::ZMachineHeader,
    zmemory::{ZMemory, ZMemoryAddress::*},
    ZMachineVersion, ZmErrorKind, ZmResult,
};
use instructions::Operation;

/// The Z-machine's processing unit.
///
/// This virtual processor is Big Endian, which means a 2-bytes word (16 bits)
/// will be processed by Most Significant Byte first (the 8 highest bits).
pub struct ZCpu {
    /// The targeted Z-machine version.
    target: ZMachineVersion,
    /// The Program Counter points to the current instruction.
    pc: u16,
}

impl ZCpu {
    pub fn from_header(header: &ZMachineHeader) -> ZmResult<Self> {
        match header.get_initial_pc() {
            Byte(pc) => Ok(ZCpu {
                target: header.get_version(),
                pc,
            }),
            _ => Err(ZmErrorKind::MemoryInvalidAddress(header.get_initial_pc()).into()),
        }
    }

    /// Fetch, decode and execute the next instruction.
    pub fn step(&mut self, memory: &mut ZMemory) -> ZmResult<()> {
        let operation = self.fetch_instruction(memory)?;
        Ok(())
    }

    fn fetch_instruction(&mut self, memory: &ZMemory) -> ZmResult<Operation> {
        Operation::decoded(self.target, || {
            let next = memory.read_byte(Byte(self.pc))?;
            self.pc = self.pc.wrapping_add(1);
            Ok(next)
        })
    }
}
