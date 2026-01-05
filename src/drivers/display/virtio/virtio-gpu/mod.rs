//! # VirtIO-GPU Module
//!
//! Driver VirtIO-GPU para displays virtualizados (QEMU/KVM).
//!
//! ## Estrutura:
//! - `protocol`: Constantes e tipos do protocolo
//! - `commands`: Estruturas de comandos/respostas
//! - `resources`: Gerenciamento de recursos GPU
//! - `state`: Estado global do dispositivo

pub mod commands;
pub mod protocol;
pub mod resources;
pub mod state;

// Re-exports públicos
pub use commands::*;
pub use protocol::*;
pub use resources::{Resource, ResourceBacking, Scanout};
pub use state::{init_state, VirtioGpuState, GPU_STATE};
