pub mod header;

use std::io::Read;

use crate::errors::{ZmErrorKind, ZmResult};
use crate::zmemory::{ZMemory, ZMemoryAddress::*};
pub use header::{ZMachineHeader, ZMachineVersion::*};

/// The core of rustif's Z-machine interpreter.
pub struct ZMachine {
    /// The virtual memory management unit.
    memory: ZMemory,
    /// The story header information, decoded from the first 64 bytes of memory.
    header: ZMachineHeader,
}

impl ZMachine {
    /// Create a new Z-machine interpreter instance and try to load the given
    /// binary source into memory and initialize the VM according to the parsed header data.
    pub fn from_story_reader(reader: &mut Read) -> ZmResult<Self> {
        let memory = ZMemory::from_story_reader(reader)?;
        let header = ZMachineHeader::from_memory(&memory)?;
        let version = header.get_version();
        println!("loaded version {}", version); // TODO: use proper logging crate
        match version {
            V3 => Ok(ZMachine { memory, header }),
            _ => Err(ZmErrorKind::MachineUnsupportedVersion(version).into()),
        }
    }
}
