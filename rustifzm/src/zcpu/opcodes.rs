/// The different instructions allowed by the Z-machine.
///
/// This internal representation allows for efficient and human-readable dispatching,
/// and will facilitate potential future tooling like a disassembler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZOpcode {
    /// 0OP:176 0 rtrue
    OP0_176,
    /// 0OP:177 1 rfalse
    OP0_177,
}

impl ZOpcode {
    // pub fn disassemble()
}
