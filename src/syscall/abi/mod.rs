//! # Syscall ABI
//!
//! Define o contrato binário para syscalls.
//!
//! ## Módulos
//! - `args`: Estrutura SyscallArgs
//! - `types`: IoVec, TimeSpec, Stat, DirEntry, PollFd
//! - `flags`: Todas as flags (IO, Map, Port, Msg, Open)
//! - `version`: Versionamento da ABI

pub mod args;
pub mod flags;
pub mod types;
pub mod version;

// Re-exports principais
pub use args::SyscallArgs;
pub use flags::*;
pub use types::*;
pub use version::{ABI_VERSION, REDSTONE_MAGIC};
