//! Estruturas Per-CPU e Load Balancing

use crate::sys::types::Tid;

/// Estrutura que representa uma CPU lógica
pub struct Cpu {
    /// ID da CPU (APIC ID)
    pub id: u32,
    /// ID da task rodando atualmente nesta CPU (se houver)
    pub current_tid: Option<Tid>,
}

impl Cpu {
    /// Cria nova estrutura de CPU
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            current_tid: None,
        }
    }
}

/// Balanceador de Carga (Load Balancer)
pub struct LoadBalancer;

impl LoadBalancer {
    /// Verifica se é necessário balancear carga entre CPUs.
    ///
    /// Em sistemas SMP, isso moveria tarefas de uma runqueue cheia para uma vazia.
    /// Atualmente (Single Core), é um no-op.
    pub fn balance() {
        // Futuro: Verificar disparidade entre runqueues[]
        // Se disparidade > threshold, migrar tasks.
    }

    /// Retorna a carga estimada do sistema (ex: número de tasks ready).
    pub fn get_load() -> u64 {
        // Retorna tamanho da runqueue global por enquanto
        super::runqueue::RUNQUEUE.lock().len() as u64
    }
}
