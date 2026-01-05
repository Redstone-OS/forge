//! # Memory Management
//!
//! Gerenciamento de memória do GPU Intel.
//! Inclui GTT (Graphics Translation Table) e alocação de buffers.

pub mod ggtt;
pub mod gtt;

pub use ggtt::Ggtt;
