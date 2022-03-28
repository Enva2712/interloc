//! Interloc is a tool to verify that changes to an interface are backwards compatible with all
//! usages of that interface

pub mod inter;
pub mod loc;
pub use inter::*;
pub use loc::*;
