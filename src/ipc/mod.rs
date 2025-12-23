//! Subsistema de Inter-Process Communication (IPC).
//!
//! O IPC do Redstone é baseado em troca de mensagens assíncronas através de Portas.
//! É o mecanismo fundamental para comunicação entre serviços, drivers e apps.

pub mod message;
pub mod port;

pub use message::Message;
pub use port::{Port, PortHandle, PortStatus};

/// Inicializa o subsistema de IPC.
pub fn init() {
    crate::kinfo!("[Init] IPC: System initialized.");
    // Futuro: Criar portas globais do sistema (ex: NameService)
}
