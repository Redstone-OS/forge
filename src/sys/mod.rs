//! # System Definitions
//!
//! Tipos e definições compartilhadas do sistema.
//!
//! ## Conteúdo
//!
//! | Módulo    | Responsabilidade                      |
//! |-----------|---------------------------------------|
//! | `error`   | Códigos de erro do kernel             |
//! | `types`   | Tipos fundamentais (Pid, Tid, etc)    |
//! | `elf`     | Estruturas ELF para loading           |

// =============================================================================
// MODULES
// =============================================================================

/// Códigos de erro do kernel
pub mod error;

/// Tipos fundamentais do sistema
pub mod types;

/// Estruturas ELF
pub mod elf;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use error::{KernelError, KernelResult};
pub use types::{Gid, Pid, Tid, Uid};
