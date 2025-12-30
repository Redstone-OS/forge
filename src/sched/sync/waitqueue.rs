//! Wait queues para bloqueio

use alloc::collections::VecDeque;
use crate::sync::Spinlock;
use crate::sys::types::Tid;

/// Wait queue - threads esperando evento
pub struct WaitQueue {
    waiters: Spinlock<VecDeque<Tid>>,
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            waiters: Spinlock::new(VecDeque::new()),
        }
    }
    
    /// Adiciona thread atual à espera
    pub fn wait(&self) {
        // TODO: pegar TID atual
        // TODO: marcar task como Blocked
        // TODO: adicionar à fila
        // TODO: chamar schedule()
    }
    
    /// Acorda uma thread
    pub fn wake_one(&self) {
        let mut waiters = self.waiters.lock();
        if let Some(_tid) = waiters.pop_front() {
            // TODO: marcar task como Ready
            // TODO: adicionar ao runqueue
        }
    }
    
    /// Acorda todas as threads
    pub fn wake_all(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(_tid) = waiters.pop_front() {
            // TODO: acordar cada uma
        }
    }
}
