//! Hardware Abstraction Layer (HAL).
//!
//! Este módulo é a ÚNICA ponte entre o Kernel Core e o Hardware.
//! Ele seleciona condicionalmente a implementação correta (x86_64, aarch64, etc).

pub mod traits;

// Seleção de Arquitetura: x86_64
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as platform;

// Re-exports globais para o kernel usar
// Exemplo: arch::cpu::halt();
pub use platform::Cpu;
pub use traits::*;
