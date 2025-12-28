//! Portas IPC (Endpoints).
//!
//! Uma Porta é uma fila de mensagens unidirecional ou bidirecional.
//! Processos escrevem em Portas para as quais têm Capability WRITE.
//! Processos leem de Portas para as quais têm Capability READ.

use super::message::Message;
use crate::sync::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;

/// Status de uma operação na Porta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortStatus {
    Ok,
    Full,
    Empty,
    Closed,
}

/// Estrutura interna da Porta.
pub struct Port {
    /// Fila de mensagens pendentes.
    queue: VecDeque<Message>,
    /// Capacidade máxima da fila (backpressure).
    capacity: usize,
    /// Se a porta está aberta para novos envios.
    active: bool,
}

/// Wrapper thread-safe para Portas (Reference Counted).
#[derive(Clone)]
pub struct PortHandle(Arc<Mutex<Port>>);

impl Port {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
            active: true,
        }
    }
}

impl PortHandle {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(Mutex::new(Port::new(capacity))))
    }

    pub fn send(&self, msg: Message) -> PortStatus {
        let msg_id = msg.header.id;
        let mut port = self.0.lock();

        if !port.active {
            crate::kwarn!("(IPC) send: Porta fechada para msg_id=", msg_id);
            return PortStatus::Closed;
        }

        if port.queue.len() >= port.capacity {
            crate::ktrace!("(IPC) send: Porta cheia. msg_id=", msg_id);
            return PortStatus::Full;
        }

        crate::ktrace!("(IPC) send: Mensagem enfileirada ID=", msg_id);
        crate::ktrace!("(IPC) send: Mensagem bytes=", msg.header.data_len as u64);
        port.queue.push_back(msg);
        PortStatus::Ok
    }

    /// Recebe uma mensagem da porta (Non-blocking).
    pub fn recv(&self) -> Result<Message, PortStatus> {
        let mut port = self.0.lock();

        if let Some(msg) = port.queue.pop_front() {
            crate::ktrace!("(IPC) recv: Mensagem retirada ID=", msg.header.id);
            Ok(msg)
        } else if !port.active {
            Err(PortStatus::Closed)
        } else {
            Err(PortStatus::Empty)
        }
    }

    /// Fecha a porta, impedindo novos envios.
    pub fn close(&self) {
        crate::kdebug!("(IPC) port: Fechando porta...");
        let mut port = self.0.lock();
        port.active = false;
    }

    /// Retorna o número de mensagens pendentes.
    pub fn pending_count(&self) -> usize {
        self.0.lock().queue.len()
    }
}
