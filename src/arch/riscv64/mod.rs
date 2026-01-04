//! Implementação RISC-V 64
//!
//! Esta camada fornece a abstração necessária para que o kernel RedstoneOS
//! funcione em hardware RISC-V de 64 bits (RV64).

pub mod cpu;
pub mod interrupts;
pub mod memory;
pub mod syscall;

pub mod smp;
pub mod vmm;

pub use cpu::Cpu;

/// Inicializa o básico da arquitetura RISC-V: CSRs, Interrupções e Syscall.
///
/// # Safety
///
/// Deve ser chamado no início do boot, single-core.
pub unsafe fn init_basics() {
    // Configura interrupções básicas (stvec, sie, sstatus)
    interrupts::init();

    // Inicializa o subsistema de syscall (ecall)
    syscall::init();

    crate::kinfo!("(Arch) RISC-V Sistema Inicializado (CSRs, Interrupts, Syscall)");
}
