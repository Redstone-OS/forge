//! # Tipos Seguros de Memória - Módulo Principal
//!
//! Tipos que expressam invariantes de memória em tempo de compilação.
//!
//! ## 🎯 Propósito
//!
//! Rust permite expressar garantias de memória via tipos. Este módulo
//! fornece abstrações que previnem erros comuns:
//!
//! - **Pinned<T>**: Garante que valor não será movido
//! - **VMO**: Virtual Memory Object com capacidades
//!
//! ## Benefícios
//!
//! - Erros detectados em tempo de compilação
//! - Contratos claros entre módulos
//! - Documentação via tipos

pub mod pinned;
pub mod vmo;

pub use pinned::{Pin, Pinned};
pub use vmo::{VMOFlags, VMOHandle, VMO};
