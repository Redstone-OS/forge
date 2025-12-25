//! Sistema de Syscalls do Redstone OS
//!
//! Arquitetura capability-based com handles.
//! Numeração própria (NÃO compatível com Linux/POSIX).
//!
//! # Módulos
//!
//! - `abi`: Convenção de chamada, estruturas (IoVec, TimeSpec)
//! - `error`: Códigos de erro (SysError)
//! - `numbers`: Constantes das syscalls
//! - `dispatch`: Dispatcher central
//! - `process`: exit, spawn, wait, yield
//! - `memory`: alloc, free, map, unmap
//! - `handle`: handle_create, dup, close
//! - `ipc`: create_port, send, recv
//! - `io`: readv, writev
//! - `time`: clock, sleep, monotonic
//! - `system`: sysinfo, debug

pub mod abi;
pub mod dispatch;
pub mod error;
pub mod numbers;

// Módulos de implementação
pub mod handle;
pub mod io;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod system;
pub mod time;

// Re-exports principais
pub use dispatch::syscall_dispatcher;
pub use error::{SysError, SysResult};
