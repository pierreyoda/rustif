use crate::{
    zmachine::ZMachineHeader,
    zmemory::{ZMemory, ZMemoryAddress},
    ZMachineVersion, ZmError, ZmResult,
};

/// A ZCharacter is encoded over 5 bits.
pub type ZCharacter = u8;

/// A string of character.
///
/// See: http://inform-fiction.org/zmachine/standards/z1point1/sect03.html
///
/// Z-machine text is a sequence of ZSCII character codes (ZSCII is a system similar to ASCII: see R3.8).
/// These ZSCII values are encoded into memory using a string of Z-characters.
pub struct ZString {
    /// R3.2: Text in memory consists of a sequence of 2-byte words. Each word is divided into three 5-bit 'Z-characters', plus 1 bit left over, arranged as
    ///
    /// ```md
    ///    --first byte-------   --second byte---
    ///    7    6 5 4 3 2  1 0   7 6 5  4 3 2 1 0
    ///    bit  --first--  --second---  --third--
    /// ````
    ///
    /// The bit is set only on the last 2-byte word of the text, and so marks the end.
    content: Vec<ZCharacter>,
}

impl ZString {
    /// Get the size of the string.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Decode the string into UTF-8.
    pub fn decode(
        &self,
        version: ZMachineVersion,
        abbreviations_table: Option<&ZAbbreviationsTable>,
    ) -> ZmResult<String> {
        let mut alphabet = ZAlphabet::A0LowerCase;
        let mut result = String::with_capacity(self.len());

        // TODO:

        Ok(result)
    }
}

/// R3.2.1: There are three 'alphabets', A0 (lower case), A1 (upper case) and A2 (punctuation)
/// and during printing one of these is current at any given time.
///
/// Initially A0 is current. The meaning of a Z-character may depend on which alphabet is current.
#[derive(Copy, Clone, Debug)]
pub enum ZAlphabet {
    A0LowerCase,
    A1UpperCase,
    A2Punctuation,
}

const A0_CHARS: &[char; 32] = &[
    ' ', ' ', ' ', ' ', ' ', ' ', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const A1_CHARS: &[char; 32] = &[
    ' ', ' ', ' ', ' ', ' ', ' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const A2_CHARS: &[char; 32] = &[
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.',
    ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')',
];
const A2_V1_CHARS: &[char; 32] = &[
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',',
    '!', '?', '_', '#', '\'', '"', '/', '\\', '<', '-', ':', '(', ')',
];

impl ZAlphabet {
    /// Returns the shifted `ZAlphabet` according to the given shifting character (V1 and V2).
    ///
    /// The shifted alphabet may be permanent (bool true) or only for the next character (bool false).
    ///
    /// R3.2.2:
    ///
    /// In Versions 1 and 2, the current alphabet can be any of the three.
    ///
    /// The Z-characters 2 and 3 are called 'shift' characters and change the alphabet for the next character only.
    ///
    /// The new alphabet depends on what the current one is:
    ///
    /// ```md
    ///             from A0  from A1  from A2
    /// Z-char 2      A1       A2       A0
    /// Z-char 3      A2       A0       A1
    /// ```
    /// Z-characters 4 and 5 permanently change alphabet, according to the same table, and are called 'shift lock' characters.
    pub fn shifted_with_maybe_lock(&self, shift_character: ZCharacter) -> ZmResult<(Self, bool)> {
        match (shift_character) {
            2 => Ok((self.next(), false)),
            3 => Ok((self.previous(), false)),
            4 => Ok((self.next(), true)),
            5 => Ok((self.previous(), true)),
            _ => Err(ZmError::StringInvalidAlphabetShiftCharacter(
                shift_character,
            )),
        }
    }

    /// Returns the shifted `ZAlphabet` according to the given shifting character (V3+) for the next character.
    ///
    /// R3.2.3:
    /// In Versions 3 and later, the current alphabet is always A0 unless changed for 1 character only:
    /// Z-characters 4 and 5 are shift characters.
    /// Thus 4 means "the next character is in A1" and 5 means "the next is in A2". There are no shift lock characters.
    pub fn shifted(shift_character: ZCharacter) -> ZmResult<Self> {
        match shift_character {
            4 => Ok(ZAlphabet::A1UpperCase),
            5 => Ok(ZAlphabet::A2Punctuation),
            _ => Err(ZmError::StringInvalidAlphabetShiftCharacter(
                shift_character,
            )),
        }
    }

    /// Get the UTF-8 character corresponding to the given `ZCharacter` and `ZMachineVersion`.
    ///
    /// R3.5.3:
    /// In Versions 2 to 4, the alphabet table for converting Z-characters into ZSCII character codes is as follows:
    ///
    /// ```md
    /// Z-char    6789abcdef0123456789abcdef
    /// current   --------------------------
    ///   A0      abcdefghijklmnopqrstuvwxyz
    ///   A1      ABCDEFGHIJKLMNOPQRSTUVWXYZ
    ///   A2       ^0123456789.,!?_#'"/\-:()
    ///           --------------------------
    /// ```
    ///
    /// (Character 6 in A2 is printed as a space here, but is not translated using the alphabet table:
    /// see S 3.4 above. Character 7 in A2, written here as a circumflex ^, is a new-line.)
    /// For example, in alphabet A1 the Z-character 12 is translated as a capital G (ZSCII character code 71).
    ///
    /// R3.5.4:
    /// Version 1 has a slightly different A2 row in its alphabet table (new-line is not needed, making
    /// room for the < character):
    ///
    /// ```md
    ///            6789abcdef0123456789abcdef
    ///           --------------------------
    ///   A2       0123456789.,!?_#'"/\<-:()
    ///           --------------------------
    /// ```
    pub fn get_character(&self, char: ZCharacter, version: ZMachineVersion) -> char {
        let char_index = char as usize;
        match (self, version) {
            (ZAlphabet::A0LowerCase, _) => A0_CHARS[char_index],
            (ZAlphabet::A1UpperCase, _) => A1_CHARS[char_index],
            (ZAlphabet::A2Punctuation, ZMachineVersion::V1) => A2_V1_CHARS[char_index],
            (ZAlphabet::A2Punctuation, _) => A2_CHARS[char_index],
        }
    }

    fn previous(&self) -> ZAlphabet {
        match self {
            ZAlphabet::A0LowerCase => ZAlphabet::A2Punctuation,
            ZAlphabet::A1UpperCase => ZAlphabet::A0LowerCase,
            ZAlphabet::A2Punctuation => ZAlphabet::A1UpperCase,
        }
    }

    fn next(&self) -> ZAlphabet {
        match self {
            ZAlphabet::A0LowerCase => ZAlphabet::A1UpperCase,
            ZAlphabet::A1UpperCase => ZAlphabet::A2Punctuation,
            ZAlphabet::A2Punctuation => ZAlphabet::A0LowerCase,
        }
    }
}

/// In V3+, Z-characters 1, 2 and 3 represent abbreviations, sometimes also called 'synonyms' (for traditional reasons):
/// the next Z-character indicates which abbreviation string to print.
///
/// If z is the first Z-character (1, 2 or 3) and x the subsequent one,
/// then the interpreter must look up entry 32(z-1)+x in the abbreviations table
/// and print the string at that word address.
///
/// In V2, Z-character 1 has this effect (but 2 and 3 do not, so there are only 32 abbreviations).
pub struct ZAbbreviationsTable {
    address: ZMemoryAddress,
}

impl ZAbbreviationsTable {
    pub fn from_memory_and_header(
        memory: &ZMemory,
        header: &ZMachineHeader,
    ) -> ZmResult<Option<Self>> {
        if header.get_version() == ZMachineVersion::V1 {
            return Ok(None);
        }
        let address = header
            .get_location_abbreviations_table()
            .expect("V2+ header should define an abbreviations table address");
        Ok(Some(Self { address }))
    }
}

/// R3.8: The character set of the Z-machine is called ZSCII
/// (Zork Standard Code for Information Interchange; pronounced to rhyme with "xyzzy").
///
/// ZSCII codes are 10-bit unsigned values between 0 and 1023.
/// Story files may only legally use the values which are defined below.
/// Note that some values are defined only for input and some only for output.
pub struct ZSCII(u16);

impl TryInto<Option<char>> for ZSCII {
    type Error = ZmError;

    fn try_into(self) -> ZmResult<Option<char>> {
        match self.0 {
            // R3.8.2.1: ZSCII code 0 ("null") is defined for output but has no effect in any output stream.
            // (It is also used as a value meaning "no character" when reporting terminating character codes,
            // but is not formally defined for input.)
            0 => Ok(None),
            // R3.8.2.5: ZSCII code 13 ("carriage return") is defined for input and output.
            13 => Ok(Some('\n')),
            // R3.8.3: ZSCII codes between 32 ("space") and 126 ("tilde") are defined for input and output,
            // and agree with standard ASCII (as well as all of the ISO 8859 character sets and Unicode).
            32..=126 => Ok(Some(self.0 as u8 as char)),
            // R3.8.5: The block of codes between 155 and 251 are the "extra characters" and are used
            // differently by different story files. Some will need accented Latin characters
            // (such as French E-acute), others unusual punctuation (Spanish question mark),
            // others new alphabets (Cyrillic or Hebrew); still others may want dingbat characters,
            // mathematical or musical symbols, and so on.
            155..=251 => Ok(Some(DEFAULT_UNICODE_TABLE[(self.0 as usize) - 155])),
            // Invalid ZSCII character
            _ => Err(ZmError::StringInvalidZSCIICharacterCode(self.0)),
        }
    }
}

/// Default Unicode characters table (Table 1, see R3.8.5.3).
const DEFAULT_UNICODE_TABLE: &[char] = &[
    'ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß', '»', '«', 'ë', 'ï', 'ÿ', 'Ë', 'Ï', 'á', 'é', 'í', 'ó', 'ú',
    'ý', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ý', 'à', 'è', 'ì', 'ò', 'ù', 'À', 'È', 'Ì', 'Ò', 'Ù', 'â', 'ê',
    'î', 'ô', 'û', 'Â', 'Ê', 'Î', 'Ô', 'Û', 'å', 'Å', 'ø', 'Ø', 'ã', 'ñ', 'õ', 'Ã', 'Ñ', 'Õ', 'æ',
    'Æ', 'ç', 'Ç', 'þ', 'ð', 'Þ', 'Ð', '£', 'œ', 'Œ', '¡', '¿',
];
