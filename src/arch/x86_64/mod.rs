//! Implementação HAL para x86_64.
//!
//! Agrupa os módulos de baixo nível. A exportação aqui define o que está
//! disponível via `crate::arch::platform::*`.

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod ports;

// Re-exporta a implementação concreta de CPU para uso genérico
pub use cpu::CpuidResult;
pub use cpu::X64Cpu as Cpu;

// Incluir Assembly do Handler de Syscall
core::arch::global_asm!(include_str!("syscall.s"));
