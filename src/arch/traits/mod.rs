//! Traits do Hardware Abstraction Layer (HAL).
//! Interfaces públicas que o Kernel Core usa para falar com o hardware.

pub mod cpu;

// Re-exportar para facilitar uso: `use crate::arch::traits::CpuOps;`
pub use cpu::CpuOps;
