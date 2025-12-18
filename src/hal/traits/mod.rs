//! Traits do HAL
//!
//! Define as interfaces abstratas para hardware.

pub mod cpu;
pub mod timer;
pub mod irq;
pub mod mmu;

pub use cpu::*;
pub use timer::*;
pub use irq::*;
pub use mmu::*;
