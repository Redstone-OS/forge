//! Hardware Abstraction Layer (HAL)
//!
//! Abstração de hardware para facilitar portabilidade.
//!
//! # Arquiteturas Suportadas
//! - x86_64: Funcional (prioridade)
//! - aarch64: TODO (estrutura básica)
//! - riscv64: TODO (estrutura básica)
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar HAL x86_64 completo
//! - TODO(prioridade=baixa, versão=v2.0): Implementar HAL aarch64
//! - TODO(prioridade=baixa, versão=v2.0): Implementar HAL riscv64

pub mod traits;
pub mod common;
pub mod platform;

pub use traits::*;
