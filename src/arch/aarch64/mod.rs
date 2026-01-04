//! Implementação ARM64 (aarch64)
//!
//! Esta camada fornece a abstração necessária para que o kernel RedstoneOS
//! funcione em hardware ARM de 64 bits.

pub mod cpu;
pub mod interrupts;
pub mod memory;
pub mod syscall;

pub mod smp;
pub mod vmm;

pub use cpu::Cpu;

/// Inicializa o básico da arquitetura ARM64: Exception Levels, GIC e Syscall.
///
/// # Safety
///
/// Deve ser chamado no início do boot, single-core.
pub unsafe fn init_basics() {
    // Configura vetores de exceção (VBAR_EL1)
    interrupts::init();

    // Inicializa o subsistema de syscall (SVC)
    syscall::init();

    crate::kinfo!("(Arch) ARM64 Sistema Inicializado (Exception Levels, GIC, Syscall)");
}
