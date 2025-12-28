//! # Kernel Library (KLib)
//!
//! Utilitários de baixo nível para o kernel.
//!
//! ## Módulos
//!
//! | Módulo          | Responsabilidade                          |
//! |-----------------|-------------------------------------------|
//! | `align`         | Alinhamento de memória (up, down)         |
//! | `bitmap`        | Gerenciamento de bits (usado pelo PMM)    |
//! | `mem_funcs`     | memset/memcpy sem SSE                     |
//! | `hash`          | Tabela hash para lookup rápido            |
//! | `list`          | Lista duplamente ligada intrusiva         |
//! | `string`        | Manipulação de strings sem std            |
//! | `tree`          | Red-Black Tree                            |

// =============================================================================
// CORE UTILITIES
// =============================================================================

/// Funções de alinhamento de memória
pub mod align;

/// Bitmap genérico
pub mod bitmap;

/// Funções de memória (memset, memcpy)
pub mod mem_funcs;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// Tabela hash
pub mod hash;

/// Lista duplamente ligada
pub mod list;

/// Manipulação de strings
pub mod string;

/// Red-Black Tree
pub mod tree;

// =============================================================================
// TEST FRAMEWORK
// =============================================================================

/// Framework de testes do kernel
pub mod test_framework;

#[cfg(feature = "self_test")]
pub mod test;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use align::{align_down, align_up, is_aligned};
pub use bitmap::Bitmap;
pub use mem_funcs::{memcpy, memset, memmove};
pub use test_framework::{TestCase, TestResult};
