use crate::{
    zmachine::ZMachineHeader,
    zmemory::{ZMemory, ZMemoryAddress},
    zstring::ZString,
    ZMachineVersion, ZmResult,
};

/// The objects table, held in dynamic memory.
///
/// It consists of a list of entries, one for each object.
/// Objects are numbered consecutively from 1 upward, with object number 0 being used to mean "nothing"
/// (though there is formally no such object).
///
/// See: http://inform-fiction.org/zmachine/standards/z1point1/sect12.html
pub struct ZObjectsTable {
    /// Stored at address 0x0A in the header (see R12.1).
    address: ZMemoryAddress,
    /// Property defaults table (see R12.2).
    ///
    /// Contains 31 words (2 bytes) from V1 to V3 included, and 63 in V4+.
    /// When the game attempts to read the value of property n for an object which
    /// does not provide property n, the n-th entry in this table is the resulting value.
    property_defaults: Vec<u16>,
}

impl ZObjectsTable {
    pub fn from_memory_and_header(memory: &ZMemory, header: &ZMachineHeader) -> ZmResult<Self> {
        // retrieve the table address
        let address = header.get_location_object_table();
        // read the property defaults table memory block
        let property_defaults_size: u16 = if header.get_version() >= ZMachineVersion::V4 {
            63
        } else {
            31
        };
        let mut property_defaults = Vec::with_capacity(property_defaults_size as usize);
        let address_as_byte = address.as_byte()?;
        for word_address in
            (address_as_byte..address_as_byte + property_defaults_size * 2).step_by(2)
        {
            property_defaults.push(memory.read_word(ZMemoryAddress::Word(word_address))?);
        }

        Ok(Self {
            address,
            property_defaults,
        })
    }
}

/// Size of objects from V1 to V3 included, in bytes.
pub const ZOBJECT_LEGACY_SIZE: u16 = 9;

/// Objects are stored in a tree-like fashion where each object has a parent,
/// a sibling (the next child of the parent) and children.
///
/// An individual object is a collection of variables: flag-like attributes and
/// numerical properties. Attributes are numbered from 0 upward and properties from
/// 1 upward.
///
/// Per R12.3, in versions 1 to 3, there are at most 255 objects, each having a 9-byte entry as follows:
///
/// ```md
/// the 32 attribute flags     parent     sibling     child   properties
/// ---32 bits in 4 bytes---   ---3 bytes------------------  ---2 bytes--
/// ```
///
/// R12.4: each object has its own property table.
/// Each of these can be anywhere in dynamic memory (indeed, a game can legally change an object's
/// properties table address in play, provided the new address points to another valid properties table).
/// The header of a property table is as follows:
///
/// ```md
///   text-length     text of short name of object
///   -----byte----   --some even number of bytes---
/// ```
///
/// where the text-length is the number of 2-byte words making up the text,
/// which is stored in the usual format. (This means that an object's short name is limited to 765 Z-characters.)
/// After the header, the properties are listed in descending numerical order.
/// (This order is essential and is not a matter of convention.)
pub struct ZObject {
    attribute_flags: u32,
    address: ZMemoryAddress,
    index: u8,
    parent_index: u8,
    sibling_index: u8,
    child_index: u8,
    text_length: Option<usize>,
    text: Option<ZString>,
    properties: Vec<ZObjectProperty>,
}

impl ZObject {
    pub fn from_memory(
        &self,
        memory: &ZMemory,
        index: u8,
        address: ZMemoryAddress,
        version: ZMachineVersion,
    ) -> ZmResult<Self> {
        let address_as_byte = address.as_byte()?;
        if version > ZMachineVersion::V3 {
            let text_length = memory.read_byte(address);
            let text = ZString::new(memory, ZMemoryAddress::Byte(address_as_byte + 1))?;
            todo!()
        } else {
            let properties = vec![
                ZObjectProperty::from_memory(
                    1,
                    ZMemoryAddress::Byte(address_as_byte + 7),
                    memory,
                    version,
                )?,
                ZObjectProperty::from_memory(
                    2,
                    ZMemoryAddress::Byte(address_as_byte + 8),
                    memory,
                    version,
                )?,
            ];
            Ok(Self {
                attribute_flags: ((memory.read_word(ZMemoryAddress::Word(address_as_byte))?
                    as u32)
                    << 16)
                    | (memory.read_word(ZMemoryAddress::Word(address_as_byte + 2))? as u32),
                address,
                index,
                parent_index: memory.read_byte(ZMemoryAddress::Byte(address_as_byte + 4))?,
                sibling_index: memory.read_byte(ZMemoryAddress::Byte(address_as_byte + 5))?,
                child_index: memory.read_byte(ZMemoryAddress::Byte(address_as_byte + 6))?,
                text_length: None,
                text: None,
                properties,
            })
        }
    }

    pub fn get_property(&self, objects_table: &ZObjectsTable) -> ZmResult<ZObjectProperty> {
        todo!()
    }
}

/// R12.4.1: In Versions 1 to 3, each property is stored as a block:
///
/// ```md
///    size byte     the actual property data
///                 ---between 1 and 8 bytes--
/// ```
///
/// where the size byte is arranged as 32 times the number of data bytes minus one, plus the property number.
/// A property list is terminated by a size byte of 0.
/// (It is otherwise illegal for a size byte to be a multiple of 32.)
///
/// R12.4.2: In Versions 4 and later, a property block instead has the form :
///
/// ```md
///    size and number       the actual property data
///   --1 or 2 bytes---     --between 1 and 64 bytes--
/// ```
///
/// The property number occupies the bottom 6 bits of the first size byte.
///
/// R12.4.2.1: If the top bit (bit 7) of the first size byte is set,
/// then there are two size-and-number bytes as follows.
///
/// In the first byte, bits 0 to 5 contain the property number;
/// bit 6 is undetermined (it is clear in all Infocom or Inform story files); bit 7 is set.
/// In the second byte, bits 0 to 5 contain the property data length, counting in bytes;
/// bit 6 is undetermined (it is set in Infocom story files, but clear in Inform ones);
/// bit 7 is always set.
///
/// R12.4.2.1.1: Standard 1.0: A value of 0 as property data length (in the second byte)
/// should be interpreted as a length of 64. (Inform can compile such properties.)
///
/// R12.4.2.2: If the top bit (bit 7) of the first size byte is clear,
/// then there is only one size-and-number byte.
///
/// Bits 0 to 5 contain the property number;
/// bit 6 is either clear to indicate a property data length of 1, or set to indicate a length of 2;
/// bit 7 is clear.
pub struct ZObjectProperty {
    address: ZMemoryAddress,
    index: u8,
    /// V1, V2 and V3: between 1 and 8 bytes of data.
    /// V4*: between 1 and 64 bytes of data.
    length: u8,
    data: Vec<u8>,
}

/// R12.4
impl ZObjectProperty {
    /// NB: parameter `property_number` starts at 1.
    pub fn from_memory(
        property_number: u8,
        address: ZMemoryAddress,
        memory: &ZMemory,
        version: ZMachineVersion,
    ) -> ZmResult<Self> {
        if version > ZMachineVersion::V3 {
            todo!()
        } else {
            let size_byte = memory.read_byte(address)?;
            let length = (size_byte - property_number) / 32;
            debug_assert!(1 <= length && length <= 8);
            let mut data = vec![];
            let address_as_byte = address.as_byte()?;
            for offset in 1..=(length as u16) {
                data.push(memory.read_byte(ZMemoryAddress::Byte(address_as_byte + offset))?);
            }
            Ok(Self {
                address,
                index: property_number,
                length,
                data,
            })
        }
    }
}
