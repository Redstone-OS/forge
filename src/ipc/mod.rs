//! # Inter-Process Communication (IPC)
//!
//! Sistema nervoso do kernel — como processos conversam.
//!
//! ## Mecanismos
//!
//! | Tipo      | Padrão    | Cópia    | Bloqueio |
//! |-----------|-----------|----------|----------|
//! | Port      | 1:N       | Sim      | Opcional |
//! | Channel   | 1:1       | Sim      | Opcional |
//! | Pipe      | 1:1       | Stream   | Sim      |
//! | SharedMem | N:N       | Zero     | Não      |
//! | Futex     | Primitive | N/A      | Sim      |
//!
//! ## Filosofia
//!
//! - **Capability-First**: Enviar/receber requer handle válido
//! - **Zero-Copy**: SharedMem para dados grandes
//! - **Async-Ready**: Integração com scheduler para blocking

// =============================================================================
// MESSAGE PASSING
// =============================================================================

/// Mensagens e envelopes
pub mod message;

/// Portas de comunicação (1:N)
pub mod port;

/// Canais bidirecionais (1:1)
pub mod channel;

pub use message::Message;
pub use port::{Port, PortHandle};
pub use channel::Channel;

// =============================================================================
// STREAMING
// =============================================================================

/// Pipes unidirecionais
pub mod pipe;

pub use pipe::Pipe;

// =============================================================================
// SHARED MEMORY
// =============================================================================

/// Memória compartilhada (zero-copy)
pub mod shm;

pub use shm::SharedMemory;

// =============================================================================
// SYNCHRONIZATION
// =============================================================================

/// Futex (Fast Userspace Mutex)
pub mod futex;

pub use futex::Futex;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o subsistema de IPC
pub fn init() {
    crate::kinfo!("(IPC) Inicializando subsistema de IPC...");
    // Futuro: criar portas globais do sistema (NameService, etc)
    crate::kinfo!("(IPC) IPC inicializado");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
