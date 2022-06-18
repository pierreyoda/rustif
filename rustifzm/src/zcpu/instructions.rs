use crate::zmemory::{ZMemory, ZMemoryAddress, ZMemoryAddress::*};
use crate::{ZMachineVersion, ZMachineVersion::*, ZmError, ZmResult};

/// The different types of operand for an operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionOperand {
    /// A 2 bytes constant.
    ConstantLarge(u16),
    /// A 1 byte constant.
    ConstantSmall(u8),
    /// A variable in the context of the current routine.
    ///
    /// 0x00 refers to the top of the stack, the [0x01, 0x0F] to the routine's local variables
    /// and [0x10, 0xFF] to the global variables.
    Variable(u8),
    /// No operand type.
    Omitted,
}

/// The expected number of operands for an operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionOperandCount {
    Fixed(u8),
    Variable,
}

/// The different forms an instruction can take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionForm {
    Long,
    Short,
    Extended,
    Variable,
}

impl InstructionForm {
    /// Determines an instruction's form from its opcode (R4.3).
    pub fn from_opcode(opcode_msb: u8, target: ZMachineVersion) -> Self {
        use InstructionForm::*;
        if opcode_msb == 0xBE && target >= V5 {
            Extended
        } else {
            match opcode_msb & 0b_1100_0000 {
                0b_1100_0000 => Variable,
                0b_1000_0000 => Short,
                _ => Long,
            }
        }
    }
}

/// A decoded instruction for the `ZCpu` to execute.
///
/// An instruction is described in memory according to the following layout,
/// with parentheses marking optional parameters:
///
/// Opcode                    1-2 bytes
/// (Types of operands)       1-2 bytes: 4 or 8 2-bit fields
/// Operands                  0 to 8, each 1-2 bytes
/// (Store variable)          1 byte
/// (Branch offset)           1-2 bytes
/// (Text to print)           encoded string with dynamic length
///
/// Each instruction has a long, short or variable form, and extended form for V5+
///
/// Reference: section 4 of the Standards Document.
/// http://inform-fiction.org/zmachine/standards/z1point1/sect04.html
#[derive(Clone, Debug)]
pub struct Operation {
    form: InstructionForm,
    opcode_number: u8,
    operands: Vec<InstructionOperand>,
}

impl Operation {
    pub fn decoded<F>(target: ZMachineVersion, mut next_byte: F) -> ZmResult<Self>
    where
        F: FnMut() -> ZmResult<u8>,
    {
        let opcode_msb = next_byte()?;
        let form = InstructionForm::from_opcode(opcode_msb, target);
        let (opcode_number, operands_count) = match form {
            InstructionForm::Short => {
                // R4.3.1
                let operands_count = match (opcode_msb & 0b_0011_0000) >> 4 {
                    0b00 => InstructionOperandCount::Fixed(0),
                    _ => InstructionOperandCount::Fixed(1),
                };
                (opcode_msb & 0b_0000_1111, operands_count)
            }
            InstructionForm::Long => (opcode_msb & 0b_0001_1111, InstructionOperandCount::Fixed(2)), // R4.3.2
            InstructionForm::Variable => {
                // R4.3.3
                let operands_count = match (opcode_msb & 0b_0010_0000) >> 5 {
                    0b0 => InstructionOperandCount::Fixed(2),
                    0b1 => InstructionOperandCount::Variable,
                    _ => unreachable!(),
                };
                (opcode_msb & 0b_0001_1111, operands_count)
            }
            InstructionForm::Extended => (next_byte()?, InstructionOperandCount::Variable), // R4.3.4
        };
        Ok(Operation {
            form,
            opcode_number,
            operands: vec![],
        })
    }
}
