//! # Port Registry
//!
//! Registry global de portas nomeadas para IPC.

use super::{Port, PortHandle, PortStatus};
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::string::String;

// ============================================================================
// PORT REGISTRY
// ============================================================================

/// ID de porta
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortId(pub u64);

impl PortId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Registry global de portas
pub struct PortRegistry {
    /// Portas por ID
    ports: BTreeMap<PortId, PortHandle>,
    /// Portas por nome (para lookup)
    named: BTreeMap<String, PortId>,
    /// Próximo ID
    next_id: u64,
}

impl PortRegistry {
    pub const fn new() -> Self {
        Self {
            ports: BTreeMap::new(),
            named: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Cria nova porta
    pub fn create(&mut self, capacity: usize) -> PortId {
        let id = PortId(self.next_id);
        self.next_id += 1;

        let handle = PortHandle::new(capacity);
        self.ports.insert(id, handle);

        id
    }

    /// Cria porta nomeada
    pub fn create_named(&mut self, name: &str, capacity: usize) -> PortId {
        let id = self.create(capacity);
        self.named.insert(String::from(name), id);
        id
    }

    /// Busca porta por nome
    pub fn lookup(&self, name: &str) -> Option<PortId> {
        self.named.get(name).copied()
    }

    /// Obtém handle da porta
    pub fn get(&self, id: PortId) -> Option<&PortHandle> {
        self.ports.get(&id)
    }

    /// Envia mensagem para porta
    pub fn send(&self, id: PortId, msg: super::super::Message) -> PortStatus {
        if let Some(handle) = self.ports.get(&id) {
            handle.send(msg)
        } else {
            PortStatus::Closed
        }
    }

    /// Recebe mensagem de porta
    pub fn recv(&self, id: PortId) -> Result<super::super::Message, PortStatus> {
        if let Some(handle) = self.ports.get(&id) {
            handle.recv()
        } else {
            Err(PortStatus::Closed)
        }
    }
}

/// Registry global (protegido por spinlock)
pub static PORT_REGISTRY: Spinlock<PortRegistry> = Spinlock::new(PortRegistry::new());
