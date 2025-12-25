//! Subsistema de Inter-Process Communication (IPC).
//!
//! O IPC do Redstone é baseado em troca de mensagens assíncronas através de Portas.
//! É o mecanismo fundamental para comunicação entre serviços, drivers e apps.

pub mod message;
pub mod port;
pub mod test;

pub use message::Message;
pub use port::{Port, PortHandle, PortStatus};

/// Inicializa o subsistema de IPC.
pub fn init() {
    crate::kinfo!("(IPC) Inicializando subsistema de mensagens...");
    crate::kdebug!("(IPC) init: Protocolo assíncrono baseado em capacidades ativo");
    // Futuro: Criar portas globais do sistema (ex: NameService)
    crate::kinfo!("(IPC) Inicializado");
}
