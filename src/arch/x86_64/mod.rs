//! Implementação HAL para x86_64.

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod ports; // Adicionado

pub use cpu::X64Cpu as Cpu;
