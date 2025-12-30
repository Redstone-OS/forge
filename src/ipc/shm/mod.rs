//! # Shared Memory (SHM)
//!
//! Memória compartilhada zero-copy entre processos.

mod shm;

pub use shm::{SharedMemory, ShmError, ShmId, SHM_REGISTRY};
