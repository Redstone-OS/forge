//! # Memory Syscalls
//!
//! Alocação e mapeamento de memória.

pub mod alloc;
pub mod brk;
pub mod madvise;
pub mod mmap;
pub mod vmo;

pub use alloc::*;
