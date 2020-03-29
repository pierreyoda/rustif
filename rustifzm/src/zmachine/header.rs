use std::convert::TryFrom;
use std::fmt;

use crate::errors::{ZmError, ZmErrorKind, ZmResult};
use crate::zmemory::{ZMemory, ZMemoryAddress, ZMemoryAddress::*};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

use ZMachineVersion::*;

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

bitflags! {
    /// Byte holding the Flags 1 game & interpreter information for V1 to V3 versions.
    pub struct ZMachineHeaderFlags1: u8 {
        /// Status line type is hours:min if set and score/turns if not.
        const STATUS_LINE_TYPE = 0b_0000_0010;
        /// Is the story file split across two discs?
        const STORY_SPLIT_DISCS = 0b_0000_0100;
        /// Is the status line unavailable?
        const STATUS_LINE_UNAVAILABLE = 0b_0001_0000;
        /// Is screen-splitting available?
        const SCREEN_SPLITTING_AVAILABLE = 0b_0010_0000;
        /// Is a variable-pitch font the default?
        const VARIABLE_PITCH_FONT_IS_DEFAULT = 0b_0100_0000;
    }
}

bitflags! {
    /// Byte holding the Flags 1 information about available features (V4+).
    pub struct ZMachineHeaderFlags1Features: u8 {
        const AVAILABLE_COLORS = 0b_0000_0001;
        /// Picture displaying available?
        const AVAILABLE_PICTURE = 0b_0000_0010;
        const AVAILABLE_BOLDFACE = 0b_0000_0100;
        const AVAILABLE_ITALIC = 0b_0000_1000;
        /// Fixed-space style available?
        const AVAILABLE_FIXED_SPACE = 0b_0001_0000;
        /// Sound effects available?
        const AVAILABLE_SFX = 0b_0010_0000;
        /// Timed keyboard input available?
        const AVAILABLE_TIMED_INPUT = 0b_1000_0000;
    }
}

impl Default for ZMachineHeaderFlags1Features {
    fn default() -> Self {
        Self::AVAILABLE_COLORS
    }
}

bitflags! {
    /// Byte holding the Flags 2 information about game requested features and global state.
    ///
    /// All request features bits must be cleared if not supported except for `REQUEST_COLORS`.
    pub struct ZMachineHeaderFlags2: u16 {
        /// Set when transcripting is on.
        ///
        /// Can be set by the game at runtime.
        const ENABLE_TRANSCRIPTING = 0b_0000_0001;
        /// Force printing in fixed-pitch font (V3+).
        ///
        /// Can be set by the game at runtime.
        const FORCE_PRINTING_FIXED_PITCH = 0b_0000_0010;
        /// Set by the interpreter to request screen redraw, cleared by the game when it complies (V6+).
        ///
        /// Can be cleared by the game at runtime.
        const MUST_REDRAW_SCREEN = 0b_0000_0100;
        /// Game wants to use pictures (V5+).
        const REQUEST_PICTURES = 0b_0000_1000;
        /// Game wants to use the UNDO opcodes (V5+).
        const REQUEST_UNDO_OPCODES = 0b_0001_0000;
        /// Game wants to use a mouse (V5+).
        const REQUEST_MOUSE = 0b_0010_0000;
        /// Game wants to use colors (V5+).
        const REQUEST_COLORS = 0b_0100_0000;
        /// Game wants to use sound effects (V5+).
        const REQUEST_SFX = 0b_1000_0000;
        /// Game wants to use menus (V6+).
        const REQUEST_MENUS = 0x0100;
    }
}

impl ZMachineHeaderFlags2 {
    /// Return all allowed bits for the given Z-machine version and available features.
    pub fn allowed_flags(
        version: ZMachineVersion,
        flags1: &Option<ZMachineHeaderFlags1Features>,
    ) -> Self {
        let base = Self::ENABLE_TRANSCRIPTING
            | Self::REQUEST_COLORS
            | if version >= V3 {
                Self::FORCE_PRINTING_FIXED_PITCH
            } else {
                Self::empty()
            };
        match flags1 {
            Some(flags1) => {
                base | if version >= V5 {
                    Self::REQUEST_UNDO_OPCODES
                        | Self::REQUEST_MOUSE
                        | if flags1.contains(ZMachineHeaderFlags1Features::AVAILABLE_PICTURE) {
                            Self::REQUEST_PICTURES
                        } else {
                            Self::empty()
                        }
                        | if flags1.contains(ZMachineHeaderFlags1Features::AVAILABLE_SFX) {
                            Self::REQUEST_SFX
                        } else {
                            Self::empty()
                        }
                } else {
                    Self::empty()
                } | if version >= V6 {
                    Self::REQUEST_MENUS
                } else {
                    Self::empty()
                }
            }
            None => base,
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
    base_high_memory: ZMemoryAddress,
    /// Initial value of the Program Counter.
    ///
    /// For V6+, packed address of the initial "main" routine.
    initial_pc: ZMemoryAddress,
    flags1_old: Option<ZMachineHeaderFlags1>,
    flags1: Option<ZMachineHeaderFlags1Features>,
    flags2: ZMachineHeaderFlags2,
    /// Location of dictionary.
    location_dictionary: ZMemoryAddress,
    /// Location of object table.
    location_object_table: ZMemoryAddress,
    /// Location of global variables table.
    location_global_variables_table: ZMemoryAddress,
    /// Base of static memory.
    base_static_memory: ZMemoryAddress,
    /// (V2+) Location of abbreviations table.
    location_abbreviations_table: Option<ZMemoryAddress>,
}

impl ZMachineHeader {
    /// Returns the decoded header information.
    pub fn from_memory(memory: &ZMemory) -> ZmResult<Self> {
        let version_raw = memory.read_byte(Byte(0x00))?;
        let version = ZMachineVersion::try_from(version_raw)?;
        let initial_pc_raw = memory.read_word(Word(0x06))?;
        Ok(ZMachineHeader {
            version,
            initial_pc: if version >= V6 {
                Packed(initial_pc_raw)
            } else {
                Byte(initial_pc_raw)
            },
            flags1_old: None,
            flags1: None,
            flags2: ZMachineHeaderFlags2::empty(),
            base_high_memory: Byte(memory.read_word(Word(0x04))?),
            location_dictionary: Byte(memory.read_word(Word(0x08))?),
            location_object_table: Byte(memory.read_word(Word(0x0A))?),
            location_global_variables_table: Byte(memory.read_word(Word(0x0C))?),
            base_static_memory: Byte(memory.read_word(Word(0x0E))?),
            location_abbreviations_table: if version >= V2 {
                Some(Byte(memory.read_word(Word(0x18))?))
            } else {
                None
            },
        })
    }

    /// Sets the needed header data to the appropriate state after a game loading, restore or restart.
    ///
    /// This means setting all values markes as "Rst" in the header format table (see R11.1).
    pub fn reset(&mut self, memory: &mut ZMemory) -> ZmResult<()> {
        // set flags 1
        let flags1_raw = memory.read_byte(Byte(0x01))?;
        if self.version >= V4 {
            self.flags1 = Some(ZMachineHeaderFlags1Features::from_bits_truncate(flags1_raw));
            memory.write_byte(Byte(0x01), self.flags1.unwrap().bits())?;
        } else {
            self.flags1_old = Some(
                ZMachineHeaderFlags1::from_bits_truncate(flags1_raw)
                    & ZMachineHeaderFlags1::STATUS_LINE_TYPE
                    & ZMachineHeaderFlags1::STORY_SPLIT_DISCS,
            );
            memory.write_byte(Byte(0x01), self.flags1_old.unwrap().bits())?;
        }
        // filter and set flags 2
        self.flags2 = ZMachineHeaderFlags2::from_bits_truncate(memory.read_word(Word(0x10))?)
            & ZMachineHeaderFlags2::allowed_flags(self.version, &self.flags1);
        memory.write_word(Word(0x10), self.flags2.bits())?;

        // mark rustifzm as following the 1.1 Z-machine Standards (R11.1.5)
        memory.write_byte(Byte(0x32), 0x1)?; // n = 1
        memory.write_byte(Byte(0x33), 0x1)?; // m = 1

        Ok(())
    }

    pub fn get_version(&self) -> ZMachineVersion {
        self.version
    }

    pub fn get_initial_pc(&self) -> ZMemoryAddress {
        self.initial_pc
    }
}
