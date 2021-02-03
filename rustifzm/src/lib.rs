pub mod errors;
pub mod zcpu;
pub mod zmachine;
pub mod zmemory;

pub use errors::{ZmError, ZmResult};
pub use zmachine::{header::ZMachineVersion, ZMachine};

#[macro_use]
extern crate bitflags;
