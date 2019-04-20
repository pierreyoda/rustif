use std::convert::TryFrom;
use std::fmt;

use crate::errors::{ZmError, ZmErrorKind, ZmResult};
use crate::zmemory::{ZMemory, ZMemoryAddress::*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZMachineVersion {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
}

impl fmt::Display for ZMachineVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "V{}", *self as u8)
    }
}

impl TryFrom<u8> for ZMachineVersion {
    type Error = ZmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use self::ZMachineVersion::*;
        // manual conversion (would be overkill to add num-derive as a dependency unless needed elsewhere)
        match value {
            1 => Ok(V1),
            2 => Ok(V2),
            3 => Ok(V3),
            4 => Ok(V4),
            5 => Ok(V5),
            6 => Ok(V6),
            7 => Ok(V7),
            8 => Ok(V8),
            _ => Err(ZmErrorKind::MachineUnknownVersion(value).into()),
        }
    }
}

/// Holds the decoded information from the loaded program's header.
///
/// Reference: section 11 of the Standards Document
/// http://inform-fiction.org/zmachine/standards/z1point1/sect11.html
pub struct ZMachineHeader {
    /// Indicates the required Z-Machine version, from 1 for V1 to 8 for V8.
    version: ZMachineVersion,
    /// Initial value of the Program Counter.
    initial_pc: usize,
}

impl ZMachineHeader {
    pub fn from_memory(memory: &ZMemory) -> ZmResult<Self> {
        let version_raw = memory.read_byte(Byte(0x00))?;
        Ok(ZMachineHeader {
            version: ZMachineVersion::try_from(version_raw)?,
            initial_pc: 0,
        })
    }

    pub fn get_version(&self) -> ZMachineVersion {
        self.version
    }
}
