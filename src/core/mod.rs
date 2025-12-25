//! Core Module
//!
//! Contém a lógica central do kernel, independente de arquitetura,
//! mas fundamental para o funcionamento do sistema.

pub mod elf;
pub mod entry;
pub mod handle;
pub mod handoff;
pub mod logging;
pub mod panic;
pub mod test;
