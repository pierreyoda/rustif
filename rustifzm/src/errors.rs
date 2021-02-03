use thiserror::Error;

use crate::zmachine::header::ZMachineVersion;
use crate::zmemory::ZMemoryAddress;

#[derive(Debug, Error)]
pub enum ZmError {
    #[error("Machine IO error: {0}")]
    MachineIO(#[from] std::io::Error),

    #[error("Unknown Z-machine version V{0}")]
    MachineUnknownVersion(u8),
    #[error("Unsupported Z-machine version {0}")]
    MachineUnsupportedVersion(ZMachineVersion),

    #[error("Invalid memory access at address {0:#X}")]
    MemoryInvalidAccess(usize),
    #[error("Invalid or unexpected memory address {0}")]
    MemoryInvalidAddress(ZMemoryAddress),
}

pub type ZmResult<T> = Result<T, ZmError>;
