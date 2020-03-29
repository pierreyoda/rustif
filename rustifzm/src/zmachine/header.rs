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
    /// Base of high memory.
    high_memory_base: usize,
    /// Initial value of the Program Counter.
    /// 
    /// V6+: Packed address of initial "main" routine.
    initial_pc: usize,
    /// Location of dictionary.
    location_dictionary: usize,
    /// Location of object table.
    location_object_table: usize,
    /// Location of global variables table.
    location_global_variables_table: usize,
    /// Base of static memory.
    static_memory_base: usize,
    /// (V2+) Location of abbreviations table.
    location_abbreviations_table: usize,
}

impl ZMachineHeader {
    pub fn from_memory(memory: &ZMemory) -> ZmResult<Self> {
        let version_raw = memory.read_byte(Byte(0x00))?;
        Ok(ZMachineHeader {
            version: ZMachineVersion::try_from(version_raw)?,
            high_memory_base: memory.read_word(Word(0x04))? as usize,
            initial_pc: memory.read_word(Word(0x06))? as usize,
            location_dictionary: memory.read_word(Word(0x08))? as usize,
            location_object_table: memory.read_word(Word(0x0A))? as usize,
            location_global_variables_table: memory.read_word(Word(0x0C))? as usize,
            static_memory_base: memory.read_word(Word(0x0E))? as usize,
            location_abbreviations_table: memory.read_word(Word(0x18))? as usize,
        })
    }

    pub fn get_version(&self) -> ZMachineVersion {
        self.version
    }
}
