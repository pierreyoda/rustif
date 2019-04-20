use std::fmt;

use failure::{Backtrace, Context, Fail};

use rustifzm::ZmError;

pub type IFtResult<T> = Result<T, IFtError>;

/// An Interactive Fiction terminal client Error.
#[derive(Debug)]
pub struct IFtError {
    context: Context<IFtErrorKind>,
}

impl Fail for IFtError {
    fn cause(&self) -> Option<&Fail> {
        self.context.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.context.backtrace()
    }
}

impl fmt::Display for IFtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.context, f)
    }
}

impl From<IFtErrorKind> for IFtError {
    fn from(kind: IFtErrorKind) -> IFtError {
        IFtError {
            context: Context::new(kind),
        }
    }
}

impl From<Context<IFtErrorKind>> for IFtError {
    fn from(context: Context<IFtErrorKind>) -> IFtError {
        IFtError { context }
    }
}

impl From<std::io::Error> for IFtError {
    fn from(error: std::io::Error) -> IFtError {
        IFtError {
            context: Context::new(IFtErrorKind::IO(error)),
        }
    }
}

impl From<ZmError> for IFtError {
    fn from(error: ZmError) -> IFtError {
        IFtError {
            context: Context::new(IFtErrorKind::ZM(error)),
        }
    }
}

#[derive(Debug, Fail)]
pub enum IFtErrorKind {
    #[fail(display = "IO error: {}", _0)]
    IO(#[fail(cause)] std::io::Error),
    #[fail(display = "Z-machine error: {}", _0)]
    ZM(#[fail(cause)] ZmError),
}
