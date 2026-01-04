//! # Kernel Statistics
//!
//! Contadores globais para monitoramento de performance.

use core::sync::atomic::{AtomicU64, Ordering};

/// Estatísticas globais do kernel.
pub struct KernelStats {
    /// Total de interrupções.
    pub interrupts: AtomicU64,
    /// Total de syscalls.
    pub syscalls: AtomicU64,
    /// Total de trocas de contexto.
    pub context_switches: AtomicU64,
    /// Total de page faults.
    pub page_faults: AtomicU64,
    /// Total de tasks criadas.
    pub tasks_spawned: AtomicU64,
    /// Total de bytes alocados.
    pub bytes_allocated: AtomicU64,
    /// Total de bytes liberados.
    pub bytes_freed: AtomicU64,
}

impl KernelStats {
    /// Cria nova instância.
    const fn new() -> Self {
        Self {
            interrupts: AtomicU64::new(0),
            syscalls: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            page_faults: AtomicU64::new(0),
            tasks_spawned: AtomicU64::new(0),
            bytes_allocated: AtomicU64::new(0),
            bytes_freed: AtomicU64::new(0),
        }
    }

    /// Incrementa contador de interrupções.
    #[inline]
    pub fn inc_interrupts(&self) {
        self.interrupts.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de syscalls.
    #[inline]
    pub fn inc_syscalls(&self) {
        self.syscalls.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de trocas de contexto.
    #[inline]
    pub fn inc_context_switches(&self) {
        self.context_switches.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de page faults.
    #[inline]
    pub fn inc_page_faults(&self) {
        self.page_faults.fetch_add(1, Ordering::Relaxed);
    }

    /// Incrementa contador de tasks criadas.
    #[inline]
    pub fn inc_tasks_spawned(&self) {
        self.tasks_spawned.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra alocação de memória.
    #[inline]
    pub fn add_allocation(&self, bytes: u64) {
        self.bytes_allocated.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Registra liberação de memória.
    #[inline]
    pub fn add_free(&self, bytes: u64) {
        self.bytes_freed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Imprime estatísticas no log.
    pub fn dump(&self) {
        crate::kinfo!("=== Kernel Statistics ===");
        crate::kinfo!("Interrupts:", self.interrupts.load(Ordering::Relaxed));
        crate::kinfo!("Syscalls:", self.syscalls.load(Ordering::Relaxed));
        crate::kinfo!(
            "Context Switches:",
            self.context_switches.load(Ordering::Relaxed)
        );
        crate::kinfo!("Page Faults:", self.page_faults.load(Ordering::Relaxed));
        crate::kinfo!("Tasks Spawned:", self.tasks_spawned.load(Ordering::Relaxed));
        crate::kinfo!(
            "Bytes Allocated:",
            self.bytes_allocated.load(Ordering::Relaxed)
        );
        crate::kinfo!("Bytes Freed:", self.bytes_freed.load(Ordering::Relaxed));
        crate::kinfo!("=========================");
    }

    /// Reseta todos os contadores.
    pub fn reset(&self) {
        self.interrupts.store(0, Ordering::Relaxed);
        self.syscalls.store(0, Ordering::Relaxed);
        self.context_switches.store(0, Ordering::Relaxed);
        self.page_faults.store(0, Ordering::Relaxed);
        self.tasks_spawned.store(0, Ordering::Relaxed);
        self.bytes_allocated.store(0, Ordering::Relaxed);
        self.bytes_freed.store(0, Ordering::Relaxed);
    }
}

/// Instância global de estatísticas.
pub static STATS: KernelStats = KernelStats::new();
