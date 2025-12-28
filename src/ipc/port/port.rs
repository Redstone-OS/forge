//! Porta de comunicação

use alloc::collections::VecDeque;
use crate::sync::Mutex;
use crate::sched::wait::WaitQueue;
use super::super::message::Message;

/// ID de porta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u64);

static NEXT_PORT_ID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Status da porta
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortStatus {
    Open,
    Closed,
}

/// Porta IPC
pub struct Port {
    id: PortId,
    status: PortStatus,
    queue: Mutex<VecDeque<Message>>,
    wait: WaitQueue,
    capacity: usize,
}

impl Port {
    /// Cria nova porta
    pub fn new(capacity: usize) -> Self {
        Self {
            id: PortId(NEXT_PORT_ID.inc()),
            status: PortStatus::Open,
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            wait: WaitQueue::new(),
            capacity,
        }
    }
    
    /// Envia mensagem (não bloqueante)
    pub fn send(&self, msg: Message) -> Result<(), IpcError> {
        if self.status != PortStatus::Open {
            return Err(IpcError::PortClosed);
        }
        
        let mut queue = self.queue.lock();
        if queue.len() >= self.capacity {
            return Err(IpcError::QueueFull);
        }
        
        queue.push_back(msg);
        drop(queue);
        
        // Acordar um waiter
        self.wait.wake_one();
        
        Ok(())
    }
    
    /// Recebe mensagem (não bloqueante)
    pub fn try_recv(&self) -> Result<Message, IpcError> {
        if self.status != PortStatus::Open {
            return Err(IpcError::PortClosed);
        }
        
        self.queue.lock()
            .pop_front()
            .ok_or(IpcError::Empty)
    }
    
    /// Recebe mensagem (bloqueante)
    pub fn recv(&self) -> Result<Message, IpcError> {
        loop {
            match self.try_recv() {
                Ok(msg) => return Ok(msg),
                Err(IpcError::Empty) => {
                    // Dormir até ter mensagem
                    self.wait.wait();
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Fecha a porta
    pub fn close(&mut self) {
        self.status = PortStatus::Closed;
        self.wait.wake_all();
    }
}

/// Erro de IPC
#[derive(Debug, Clone, Copy)]
pub enum IpcError {
    PortClosed,
    QueueFull,
    Empty,
    InvalidHandle,
    PermissionDenied,
}

/// Handle para porta
#[derive(Debug, Clone, Copy)]
pub struct PortHandle(crate::core::object::Handle);
