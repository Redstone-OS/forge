//! # Kernel Library (klib)
//!
//! Biblioteca interna de utilitários `no_std` do RedstoneOS.
//! Fornece estruturas de dados, algoritmos e primitivas que não estão
//! disponíveis em `core` ou `alloc`, ou que precisam de implementação
//! especializada para ambiente bare metal.
//!
//! ## Módulos:
//!
//! | Módulo        | Descrição                                    |
//! |---------------|----------------------------------------------|
//! | `primitives`  | Utilitários de baixo nível (align, bits, mem)|
//! | `collections` | Estruturas especializadas (bitmap, intrusive)|
//! | `hash`        | Funções de hash para no_std                  |
//! | `cstr`        | Strings estilo C (strlen, strcmp)            |
//! | `bitflags`    | Macro para flags type-safe                   |
//!
//! ## Política de Zero Dependências Externas
//!
//! O klib não pode ter dependências de terceiros.
//! Todo código deve ser escrito internamente.
//!
//! ## Convenções:
//!
//! - **Sem Panic**: Retorne `Result` ou `Option`
//! - **Const quando possível**: Para uso em contextos const
//! - **Inline para hot path**: Funções críticas de performance

pub mod collections;
pub mod cstr;
pub mod hash;
pub mod primitives;

#[macro_use]
pub mod bitflags;

// Re-exports principais
pub use collections::bitmap::Bitmap;
pub use collections::intrusive::{IntrusiveList, Linked};
pub use hash::fnv::{fnv1a_hash, FnvHasher};
pub use primitives::align::{align_down, align_up, is_aligned};
pub use primitives::bits;
pub use primitives::mem;
