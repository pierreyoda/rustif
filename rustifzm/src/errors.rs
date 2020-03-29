use std::fmt;

use failure::{Backtrace, Context, Fail};

use crate::zmachine::header::ZMachineVersion;
use crate::zmemory::ZMemoryAddress;

pub type ZmResult<T> = Result<T, ZmError>;

/// A Z-machine Interpreter Error.
#[derive(Debug)]
pub struct ZmError {
    context: Context<ZmErrorKind>,
}

impl ZmError {
    pub fn kind(&self) -> &ZmErrorKind {
        self.context.get_context()
    }
}

impl Fail for ZmError {
    fn cause(&self) -> Option<&Fail> {
        self.context.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.context.backtrace()
    }
}

impl fmt::Display for ZmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.context, f)
    }
}

impl From<ZmErrorKind> for ZmError {
    fn from(kind: ZmErrorKind) -> ZmError {
        ZmError {
            context: Context::new(kind),
        }
    }
}

impl From<Context<ZmErrorKind>> for ZmError {
    fn from(context: Context<ZmErrorKind>) -> ZmError {
        ZmError { context }
    }
}

impl From<std::io::Error> for ZmError {
    fn from(error: std::io::Error) -> ZmError {
        ZmError {
            context: Context::new(ZmErrorKind::MachineIO(error)),
        }
    }
}

#[derive(Debug, Fail)]
pub enum ZmErrorKind {
    #[fail(display = "Machine IO error: {}", _0)]
    MachineIO(#[fail(cause)] std::io::Error),
    #[fail(display = "Unknown Z-machine version V{}", _0)]
    MachineUnknownVersion(u8),
    #[fail(display = "Unsupported Z-machine version {}", _0)]
    MachineUnsupportedVersion(ZMachineVersion),
    #[fail(display = "Invalid memory access at address {:#X}", _0)]
    MemoryInvalidAccess(usize),
    #[fail(display = "Invalid or unexpected memory address {}", _0)]
    MemoryInvalidAddress(ZMemoryAddress),
}
