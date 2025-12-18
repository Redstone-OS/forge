//! Drivers Legados
//!
//! Drivers para dispositivos legados (PS/2, Serial, VGA).
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Migrar de devices/ e drivers/

pub mod serial;
pub mod ps2;
pub mod vga;

pub use serial::*;
pub use ps2::*;
pub use vga::*;
