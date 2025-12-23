// (FASE2) src/drivers/mod.rs
//! Módulo de Drivers.
//!
//! Re-exporta os drivers disponíveis para o kernel.

pub mod console; // Framebuffer Text Console
pub mod pic;
pub mod serial; // UART 16550 (Logs)
pub mod timer; // PIT 8254 // 8259 PIC
pub mod video;

// Futuro:
// pub mod keyboard;
// pub mod pci;
