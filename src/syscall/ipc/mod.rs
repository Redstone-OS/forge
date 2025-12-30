//! # IPC Syscalls
//!
//! Comunicação entre processos via portas.

pub mod port;
pub mod shm;

pub use port::*;
pub use shm::*;
