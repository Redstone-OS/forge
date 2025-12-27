//! # Kernel Library (KLib)
//!
//! A `klib` é uma coleção de utilitários de baixo nível, agnósticos de arquitetura,
//! que complementam a `core` library do Rust para ambientes bare-metal.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Algoritmos Básicos:** Bitmaps, Listas, Alinhamento de memória.
//! - **Runtime functions:** Implementações de `memcpy`, `memset`.
//! - **Helpers:** Funções `const` para cálculo de endereços.
//! - **Test Framework:** Estruturas para self-tests padronizados.
//!
//! ## 🏗️ Arquitetura dos Módulos
//!
//! | Módulo           | Responsabilidade                                      |
//! |------------------|-------------------------------------------------------|
//! | `align`          | Funções de alinhamento (`align_up`, `align_down`)     |
//! | `bitmap`         | Gerenciamento de bits (usado pelo PMM)                |
//! | `mem_funcs`      | Implementação de `memset/memcpy` sem SSE              |
//! | `test_framework` | Macros e estruturas para self-tests                   |
//!
//! ## Nota sobre SSE
//!
//! SSE foi **desabilitado** no target spec (`x86_64-redstone.json`).
//! O compilador não gera instruções SSE/AVX, então `mem_funcs` agora é seguro.

// =============================================================================
// MÓDULOS
// =============================================================================

/// Funções de alinhamento de memória.
pub mod align;

/// Bitmap genérico para gerenciamento de bits.
pub mod bitmap;

/// Implementações de memset/memcpy sem SSE.
pub mod mem_funcs;

/// Framework de testes do kernel.
pub mod test_framework;

/// Testes da klib.
pub mod test;

// =============================================================================
// RE-EXPORTS PÚBLICOS
// =============================================================================

// Funções de alinhamento (API principal)
pub use align::{align_down, align_down_u64, align_up, align_up_u64, is_aligned, is_aligned_u64};

// Test framework (para uso em outros módulos)
pub use test_framework::{run_test_suite, TestCase, TestResult};
