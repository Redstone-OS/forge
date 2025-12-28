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
//! ## Estrutura
//!
//! ```text
//! arch/
//! ├── traits/     → Contratos abstratos (Cpu, Mmu, Irq)
//! └── x86_64/     → Implementação para Intel/AMD 64-bit
//!     ├── cpu.rs      → MSRs, CR0-4, CPUID
//!     ├── gdt.rs      → Segmentos de memória
//!     ├── idt.rs      → Tabela de interrupções
//!     ├── apic/       → LAPIC, IOAPIC
//!     ├── acpi/       → Tabelas ACPI (MADT, FADT)
//!     └── iommu/      → Intel VT-d
//! ```
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

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as platform;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Tamanho de página padrão (4KB para x86_64)
#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE: usize = 4096;

/// Bits de shift para converter bytes <-> páginas
#[cfg(target_arch = "x86_64")]
pub const PAGE_SHIFT: usize = 12;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use platform::Cpu;
pub use traits::cpu::CpuTrait;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
