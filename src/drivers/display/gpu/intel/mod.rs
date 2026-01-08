//! # Intel Integrated Graphics Driver (i915-style)
//!
//! Driver genérico para GPUs Intel integradas.
//! Suporta Gen9+ (Skylake, Apollo Lake, Kaby Lake, etc).
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                 Intel GPU Driver                │
//! ├─────────────────┬───────────────────────────────┤
//! │     Display     │           Memory              │
//! │   (pipe/plane)  │         (GTT/GEM)             │
//! ├─────────────────┴───────────────────────────────┤
//! │                   Hardware (hw/)                │
//! │              MMIO, Registers, Gen9              │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Device IDs Suportados
//!
//! | Device ID | Nome               | Geração  |
//! |-----------|--------------------|----------|
//! | 0x5A85    | HD Graphics 500    | Gen9 LP  |
//! | 0x5A84    | HD Graphics 505    | Gen9 LP  |
//! | 0x3184    | UHD Graphics 600   | Gen9.5   |
//! | 0x3185    | UHD Graphics 605   | Gen9.5   |

pub mod device;
pub mod display;
pub mod hw;
pub mod memory;
pub mod pci;

use alloc::sync::Arc;

// Re-exports
pub use device::IntelDevice;

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o driver Intel GPU.
///
/// Registra o driver no RDS. O matching com dispositivos é automático.
pub fn init() {
    // crate::kdebug!("(Intel GPU) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(pci::IntelGpuDriver));
}
