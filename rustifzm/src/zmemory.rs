use std::fmt;
use std::io::Read;

use crate::{errors::ZmError, ZmResult};

/// The different kinds of addresses in the Z-machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZMemoryAddress {
    /// An individual byte by absolute address.
    Byte(u16),
    /// A word by absolute address, composed of two bytes.
    ///
    /// The Z-machine is Big Endian so (address) points to the Most Significant Byte,
    /// and (address + 1) to the Least Significant Byte.
    Word(u16),
    /// A word in the bottom 128K of memory equal to twice the address value.
    ///
    /// Only used in the abbreviations table.
    RelativeWord(u16),
    /// The packed relative location of a routine or string in high memory.
    Packed(u16),
}

use self::ZMemoryAddress::*;

impl ZMemoryAddress {
    pub fn as_byte(&self) -> ZmResult<u16> {
        match self {
            Byte(address) => Ok(*address),
            _ => Err(ZmError::MemoryInvalidAddress(*self)),
        }
    }

    pub fn offset_byte(&self, offset: u16) -> ZmResult<Self> {
        match self {
            Byte(address) => Ok(ZMemoryAddress::Byte(address.wrapping_add(offset))),
            _ => Err(ZmError::MemoryInvalidAddress(*self)),
        }
    }

    pub fn offset_word(&self, offset: u16) -> ZmResult<Self> {
        match self {
            Byte(address) | Word(address) => Ok(ZMemoryAddress::Word(address.wrapping_add(offset))),
            _ => Err(ZmError::MemoryInvalidAddress(*self)),
        }
    }
}

impl fmt::Display for ZMemoryAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Byte(address) => write!(f, "ZMemoryAddress Byte = {:#X}", address),
            Word(address) => write!(f, "ZMemoryAddress Word = {:#X}", address),
            RelativeWord(address) => write!(f, "ZMemoryAddress RelativeWord = {:#X}", address),
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
    /// - dynamic memory: starts at 0x00 and ends right before the start of static memory.
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
    pub fn from_story_reader(reader: &mut dyn Read) -> ZmResult<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(ZMemory { buffer })
    }

    pub fn read_byte(&self, address: ZMemoryAddress) -> ZmResult<u8> {
        match address {
            Byte(a) => self
                .buffer
                .get(a as usize)
                .cloned()
                .ok_or(ZmError::MemoryInvalidAccess(a as usize)),
            _ => Err(ZmError::MemoryInvalidAddress(address)),
        }
    }

    pub fn read_word(&self, address: ZMemoryAddress) -> ZmResult<u16> {
        match address {
            Word(a) => {
                let upper = self
                    .buffer
                    .get(a as usize)
                    .ok_or(ZmError::MemoryInvalidAccess(a as usize))?;
                let lower = self
                    .buffer
                    .get((a + 1) as usize)
                    .ok_or(ZmError::MemoryInvalidAccess((a + 1) as usize))?;
                Ok(((*upper as u16) << 8) | (*lower as u16))
            }
            _ => Err(ZmError::MemoryInvalidAddress(address)),
        }
    }

    pub fn write_byte(&mut self, address: ZMemoryAddress, value: u8) -> ZmResult<()> {
        match address {
            Byte(a) => self
                .buffer
                .get_mut(a as usize)
                .map(|v| {
                    *v = value;
                })
                .ok_or(ZmError::MemoryInvalidAccess(a as usize)),
            _ => Err(ZmError::MemoryInvalidAddress(address)),
        }
    }

    pub fn write_word(&mut self, address: ZMemoryAddress, value: u16) -> ZmResult<()> {
        match address {
            Word(a) => {
                self.write_byte(Byte(a), ((value & 0xFF00) >> 8) as u8)?;
                self.write_byte(Byte(a + 1), (value & 0x00FF) as u8)?;
                Ok(())
            }
            _ => Err(ZmError::MemoryInvalidAddress(address)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_memory() -> ZMemory {
        ZMemory {
            buffer: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        }
    }

    #[test]
    fn test_read_byte() {
        let memory = init_memory();
        assert_eq!(memory.read_byte(ZMemoryAddress::Byte(0x01)).unwrap(), 0x02);
        assert_eq!(memory.read_byte(ZMemoryAddress::Byte(0x05)).unwrap(), 0x06);
        assert!(memory.read_byte(ZMemoryAddress::Byte(0x06)).is_err());
    }

    #[test]
    fn test_read_word() {
        let memory = init_memory();
        assert_eq!(
            memory.read_word(ZMemoryAddress::Word(0x01)).unwrap(),
            0x0203
        );
        assert_eq!(
            memory.read_word(ZMemoryAddress::Word(0x04)).unwrap(),
            0x0506
        );
        assert!(memory.read_word(ZMemoryAddress::Word(0x05)).is_err());
    }
}
