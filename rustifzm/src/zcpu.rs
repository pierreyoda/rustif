mod instructions;
mod opcodes;

use crate::{
    zmachine::ZMachineHeader,
    zmemory::{ZMemory, ZMemoryAddress::*},
    ZMachineVersion, ZmError, ZmResult,
};
use instructions::Operation;

/// The Z-machine's processing unit.
///
/// This virtual processor is Big Endian, which means a 2-bytes word (16 bits)
/// will be processed by Most Significant Byte first (the 8 highest bits).
pub struct ZCpu {
    /// The targeted Z-machine version.
    target: ZMachineVersion,
    /// The Program Counter points to the absolute address of the current instruction.
    pc: u16,
}

impl ZCpu {
    pub fn from_header(header: &ZMachineHeader) -> ZmResult<Self> {
        match header.get_initial_pc() {
            Byte(pc) => Ok(ZCpu {
                target: header.get_version(),
                pc,
            }),
            _ => Err(ZmError::MemoryInvalidAddress(header.get_initial_pc())),
        }
    }

    /// Fetch, decode and execute the next instruction.
    pub fn step(&mut self, memory: &mut ZMemory) -> ZmResult<()> {
        let operation = self.fetch_decoded_instruction(memory)?;
        self.execute_decoded_instruction(memory, &operation)?;
        Ok(())
    }

    fn fetch_decoded_instruction(&mut self, memory: &ZMemory) -> ZmResult<Operation> {
        Operation::decoded(self.target, || {
            let next = memory.read_byte(Byte(self.pc))?;
            self.pc = self.pc.wrapping_add(1);
            Ok(next)
        })
    }

    fn execute_decoded_instruction(
        &mut self,
        memory: &mut ZMemory,
        operation: &Operation,
    ) -> ZmResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
