//! # Hardware Abstraction Layer (HAL)
//!
//! Única ponte entre kernel core e hardware físico.
//!
//! ## Responsabilidades
//!
//! - Isolar código específico de arquitetura
//! - Definir traits abstratos para CPU, MMU, Interrupts
//! - Expor implementação da plataforma atual
//!
//! ## Uso
//!
//! ```ignore
//! use crate::arch::{Cpu, PAGE_SIZE};
//!
//! Cpu::disable_interrupts();
//! Cpu::halt();
//! ```

// =============================================================================
// TRAITS (CONTRATOS ABSTRATOS)
// =============================================================================

pub mod traits;

// =============================================================================
// PLATFORM SELECTION
// =============================================================================

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64 as platform;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64 as platform;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as platform;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Tamanho de página padrão (4KB para aarch64)
#[cfg(target_arch = "aarch64")]
pub const PAGE_SIZE: usize = 4096;

/// Bits de shift para converter bytes <-> páginas
#[cfg(target_arch = "aarch64")]
pub const PAGE_SHIFT: usize = 12;

/// Tamanho de página padrão (4KB para riscv64)
#[cfg(target_arch = "riscv64")]
pub const PAGE_SIZE: usize = 4096;

/// Bits de shift para converter bytes <-> páginas
#[cfg(target_arch = "riscv64")]
pub const PAGE_SHIFT: usize = 12;

/// Tamanho de página padrão (4KB para x86_64)
#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE: usize = 4096;

/// Bits de shift para converter bytes <-> páginas
#[cfg(target_arch = "x86_64")]
pub const PAGE_SHIFT: usize = 12;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use platform::init_basics;
pub use platform::Cpu;
pub use traits::cpu::CpuTrait;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
