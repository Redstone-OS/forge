//! # Hardware Abstraction Layer
//!
//! Componentes de baixo nível para acesso ao hardware Intel GPU.
//!
//! ## Módulos
//!
//! - `mmio` - Leitura/escrita de registros MMIO
//! - `regs` - Definições de registros
//! - `gen9` - Specifics para Gen9 LP (Apollo Lake)

pub mod gen9;
pub mod mmio;
pub mod regs;

pub use mmio::Mmio;
