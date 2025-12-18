//! Scheduler do Kernel (CFS - Completely Fair Scheduler)
//!
//! Implementação do scheduler baseado no CFS do Linux, que garante
//! distribuição justa de tempo de CPU entre todos os processos.
//!
//! # Arquitetura
//! - Red-Black Tree para organizar processos por vruntime
//! - 140 níveis de prioridade (0-139)
//! - Preempção baseada em tempo virtual
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar CFS completo
//! - TODO(prioridade=alta, versão=v1.0): Implementar red-black tree para runqueue
//! - TODO(prioridade=média, versão=v1.0): Adicionar suporte a prioridades
//! - TODO(prioridade=média, versão=v2.0): Implementar load balancing entre CPUs
//! - TODO(prioridade=baixa, versão=v2.0): Adicionar scheduler classes (RT, IDLE)

/// Número de níveis de prioridade (estilo Linux)
pub const PRIORITY_LEVELS: usize = 140;

/// Prioridade padrão
pub const DEFAULT_PRIORITY: u8 = 120;

/// TODO(prioridade=alta, versão=v1.0): Implementar estrutura do scheduler
pub struct Scheduler {
    // Red-black tree com processos ordenados por vruntime
    // runqueue: RBTree<Process>,
}

impl Scheduler {
    /// Cria um novo scheduler
    ///
    /// # TODOs
    /// - TODO(prioridade=alta, versão=v1.0): Implementar inicialização
    pub fn new() -> Self {
        todo!("Implementar Scheduler::new()")
    }

    /// Seleciona próximo processo para executar
    ///
    /// # TODOs
    /// - TODO(prioridade=alta, versão=v1.0): Implementar seleção CFS
    pub fn pick_next() -> Option<ProcessId> {
        todo!("Implementar pick_next() com CFS")
    }
}

/// ID de processo (placeholder)
///
/// # TODOs
/// - TODO(prioridade=alta, versão=v1.0): Mover para core/process/
pub type ProcessId = usize;
