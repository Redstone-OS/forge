//! Thread Control Block

use crate::sys::types::Tid;
use crate::mm::VirtAddr;
use super::state::TaskState;
use super::super::context::CpuContext;

/// Task ID counter
static NEXT_TID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Thread Control Block
pub struct Task {
    /// ID único
    pub tid: Tid,
    /// Estado atual
    pub state: TaskState,
    /// Contexto de CPU salvo
    pub context: CpuContext,
    /// Stack pointer do kernel
    pub kernel_stack: VirtAddr,
    /// Stack pointer do usuário
    pub user_stack: VirtAddr,
    /// Prioridade (0 = maior)
    pub priority: u8,
    /// Nome (debug)
    pub name: [u8; 32],
}

impl Task {
    /// Cria nova task
    pub fn new(name: &str) -> Self {
        let tid = Tid::new(NEXT_TID.inc() as u32);
        let mut name_buf = [0u8; 32];
        let len = name.len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        Self {
            tid,
            state: TaskState::Created,
            context: CpuContext::new(),
            kernel_stack: VirtAddr::new(0),
            user_stack: VirtAddr::new(0),
            priority: 128, // Prioridade média
            name: name_buf,
        }
    }
    
    /// Marca como pronta
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready;
    }
    
    /// Marca como bloqueada
    pub fn set_blocked(&mut self) {
        self.state = TaskState::Blocked;
    }
}
