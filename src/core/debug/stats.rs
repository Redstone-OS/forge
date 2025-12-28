/// Arquivo: core/debug/stats.rs
///
/// Propósito: Contadores estatísticos globais do kernel.
/// Usado para monitoramento de performance e diagnóstico de comportamento do sistema.
///
/// Detalhes de Implementação:
/// - Usa atômicos (AtomicU64) para permitir atualizações concorrentes sem locks (baixo overhead).
/// - Contadores monotônicos crescentes.
// Estatísticas de Debugrnel
use core::sync::atomic::{AtomicU64, Ordering};

pub struct KernelStats {
    pub interrupts: AtomicU64,
    pub syscalls: AtomicU64,
    pub context_switches: AtomicU64,
    pub page_faults: AtomicU64,
    pub tasks_spawned: AtomicU64,
}

impl KernelStats {
    const fn new() -> Self {
        Self {
            interrupts: AtomicU64::new(0),
            syscalls: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            page_faults: AtomicU64::new(0),
            tasks_spawned: AtomicU64::new(0),
        }
    }

    /// Incrementa contador de interrupções
    #[inline]
    pub fn inc_interrupts(&self) {
        self.interrupts.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de chamadas de sistema
    #[inline]
    pub fn inc_syscalls(&self) {
        self.syscalls.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de trocas de contexto
    #[inline]
    pub fn inc_context_switches(&self) {
        self.context_switches.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de page faults
    #[inline]
    pub fn inc_page_faults(&self) {
        self.page_faults.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de tarefas criadas
    #[inline]
    pub fn inc_tasks_spawned(&self) {
        self.tasks_spawned.fetch_add(1, Ordering::Relaxed);
    }

    /// Imprime estatísticas no log
    pub fn dump(&self) {
        crate::kinfo!("--- Estatísticas do Kernel ---");
        crate::kinfo!(
            "Interrupções:     ",
            self.interrupts.load(Ordering::Relaxed)
        );
        crate::kinfo!("Syscalls:         ", self.syscalls.load(Ordering::Relaxed));
        crate::kinfo!(
            "Trocas Contexto:  ",
            self.context_switches.load(Ordering::Relaxed)
        );
        crate::kinfo!(
            "Page Faults:      ",
            self.page_faults.load(Ordering::Relaxed)
        );
        crate::kinfo!(
            "Tarefas Criadas:  ",
            self.tasks_spawned.load(Ordering::Relaxed)
        );
        crate::kinfo!("--------------------");
    }
}

/// Instância global de estatísticas
pub static STATS: KernelStats = KernelStats::new();
