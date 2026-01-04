//! # System Definitions
//!
//! Tipos fundamentais e definições compartilhadas do kernel.
//!
//! ## Conteúdo
//!
//! | Módulo  | Responsabilidade                      |
//! |---------|---------------------------------------|
//! | `types` | Tipos fundamentais (Pid, Tid, etc)    |
//! | `error` | Códigos de erro do kernel             |
//! | `elf`   | Estruturas ELF para loading           |
//!
//! ## Filosofia
//!
//! Usamos **NewTypes** para garantir segurança em tempo de compilação:
//! ```text
//! kill(pid, signal)  ✓ Compila
//! kill(signal, pid)  ✗ Erro de tipo!
//! ```

pub mod elf;
pub mod error;
pub mod types;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use error::{KernelError, KernelResult};
pub use types::{FileOffset, Size, Time};
pub use types::{Gid, Pid, Tid, Uid};
