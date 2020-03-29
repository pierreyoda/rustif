pub mod header;

use std::io::Read;

use crate::{
    zcpu::ZCpu,
    zmemory::{ZMemory, ZMemoryAddress::*},
    ZmErrorKind, ZmResult,
};
pub use header::{ZMachineHeader, ZMachineVersion::*};

/// The core of rustif's Z-machine interpreter.
pub struct ZMachine {
    /// The virtual memory management unit.
    memory: ZMemory,
    /// The story header information, decoded from the first 64 bytes of memory.
    header: ZMachineHeader,
    /// The virtual processing unit.
    cpu: ZCpu,
}

impl ZMachine {
    /// Create a new Z-machine interpreter instance and try to load the given
    /// binary source into memory and initialize the VM according to the parsed header data.
    pub fn from_story_reader(reader: &mut Read) -> ZmResult<Self> {
        let mut memory = ZMemory::from_story_reader(reader)?;
        let mut header = ZMachineHeader::from_memory(&memory)?;
        let version = header.get_version();
        println!("loaded version {}", version); // TODO: use proper logging crate
        header.reset(&mut memory)?;
        let cpu = ZCpu::from_header(&header)?;
        match version {
            V3 => Ok(ZMachine {
                memory,
                header,
                cpu,
            }),
            _ => Err(ZmErrorKind::MachineUnsupportedVersion(version).into()),
        }
    }

    pub fn step(&mut self) -> ZmResult<()> {
        self.cpu.step(&mut self.memory)
    }
}
