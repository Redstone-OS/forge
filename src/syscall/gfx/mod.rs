//! # Syscalls de Gráficos / Input
//!
//! Expõe framebuffer e dispositivos de input para userspace.

pub mod framebuffer;
pub mod input;

pub use framebuffer::*;
pub use input::*;
