//! Implementação x86_64

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod ports;
pub mod syscall;

pub mod acpi;
pub mod apic;
pub mod iommu;

pub use cpu::Cpu;
