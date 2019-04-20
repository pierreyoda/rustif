use std::fmt;
use std::io::Read;

use crate::errors::{ZmErrorKind, ZmResult};

/// The different kinds of addresses in the Z-machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZMemoryAddress {
    /// An individual byte by absolute address.
    Byte(u16),
    /// A word in the bottom 128K of memory equal to twice the address value.
    ///
    /// Only used in the abbreviations table.
    Word(u16),
    /// The packed relative location of a routine or string in high memory.
    Packed(u16),
}

use self::ZMemoryAddress::*;

impl fmt::Display for ZMemoryAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Byte(address) => write!(f, "ZMemoryAddress Byte = {:#X}", address),
            Word(address) => write!(f, "ZMemoryAddress Word = {:#X}", address),
            Packed(address) => write!(f, "ZMemoryAddress Packed = {:#X}", address),
        }
    }
}

/// The Z-machine's memory management unit.
///
/// Reference: section 1 of the Standards Document
/// http://inform-fiction.org/zmachine/standards/z1point1/sect01.html
pub struct ZMemory {
    /// The raw array of bytes, which is divided into 3 regions:
    /// - dynamic memory: starts at 0x00 and ends at the start of static memory.
    ///   Must contains at least 64 bytes for the header (which ends at 0x40).
    ///   Unrestricted access for games.
    /// - static memory: starts at the address specified in the header up to either
    ///   the last byte of the story file or 0xFFFF (whichever is lower).
    ///   Read-only for games.
    /// - high memory: starts at the "high memory mark" specified in the header and continues
    ///   to the end of the story file. May overlap with static memory.
    ///   Unaccessible directly from games since strings and routines are stored here.
    buffer: Vec<u8>,
}

impl ZMemory {
    pub fn from_story_reader(reader: &mut Read) -> ZmResult<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(ZMemory { buffer })
    }

    pub fn read_byte(&self, address: ZMemoryAddress) -> ZmResult<u8> {
        match address {
            Byte(a) => self
                .buffer
                .get(a as usize)
                .map(|v| *v)
                .ok_or(ZmErrorKind::MemoryInvalidAccess(a as usize).into()),
            _ => Err(ZmErrorKind::MemoryInvalidAddress(address).into()),
        }
    }
}
