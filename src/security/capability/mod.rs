//! Capability Module
//!
//! Exposes Capability primitives and Rights.

pub mod cap;
pub mod cspace;
pub mod rights; // It was in the directory listing

pub use cap::{CapHandle, CapType, Capability};
pub use cspace::{CSpace, CapError};
pub use rights::CapRights;
