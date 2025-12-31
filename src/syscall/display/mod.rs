//! # Display Syscalls
//!
//! Syscalls para acesso ao subsistema de display e input.

pub mod buffer;
pub mod display;
pub mod input;

pub use buffer::*;
pub use display::*;
pub use input::*;
