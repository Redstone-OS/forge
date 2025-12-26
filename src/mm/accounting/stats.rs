//! # Estatísticas de Memória por Subsistema
//!
//! Contadores atômicos e funções de relatório.

use super::subsystem::Subsystem;
use core::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// ESTATÍSTICAS POR SUBSISTEMA
// =============================================================================

/// Estatísticas de uso de memória de um subsistema
#[repr(C, align(64))] // Evita false sharing
pub struct SubsystemStats {
    /// Bytes atualmente alocados
    allocated: AtomicUsize,
    /// Número de alocações
    alloc_count: AtomicUsize,
    /// Número de liberações
    free_count: AtomicUsize,
    /// Pico de uso (bytes)
    peak: AtomicUsize,
    /// Quota em bytes (0 = sem limite)
    quota: AtomicUsize,
    /// Alocações negadas por quota
    quota_denials: AtomicUsize,
}

impl SubsystemStats {
    pub const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            alloc_count: AtomicUsize::new(0),
            free_count: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            quota: AtomicUsize::new(0),
            quota_denials: AtomicUsize::new(0),
        }
    }

    /// Registra alocação
    ///
    /// Retorna false se exceder quota (alocação deve ser negada).
    pub fn record_alloc(&self, bytes: usize) -> bool {
        let quota = self.quota.load(Ordering::Relaxed);
        let current = self.allocated.load(Ordering::Relaxed);

        // Verificar quota
        if quota > 0 && current + bytes > quota {
            self.quota_denials.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        // Atualizar contadores
        let new_total = self.allocated.fetch_add(bytes, Ordering::Relaxed) + bytes;
        self.alloc_count.fetch_add(1, Ordering::Relaxed);

        // Atualizar pico
        loop {
            let peak = self.peak.load(Ordering::Relaxed);
            if new_total <= peak {
                break;
            }
            if self
                .peak
                .compare_exchange_weak(peak, new_total, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        true
    }

    /// Registra liberação
    pub fn record_free(&self, bytes: usize) {
        self.allocated.fetch_sub(bytes, Ordering::Relaxed);
        self.free_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Define quota
    pub fn set_quota(&self, bytes: usize) {
        self.quota.store(bytes, Ordering::Relaxed);
    }

    /// Bytes atualmente alocados
    pub fn allocated_bytes(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Número de alocações
    pub fn allocation_count(&self) -> usize {
        self.alloc_count.load(Ordering::Relaxed)
    }

    /// Número de liberações
    pub fn free_count(&self) -> usize {
        self.free_count.load(Ordering::Relaxed)
    }

    /// Pico de uso
    pub fn peak_bytes(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    /// Quota atual
    pub fn quota_bytes(&self) -> usize {
        self.quota.load(Ordering::Relaxed)
    }

    /// Alocações negadas por quota
    pub fn denials(&self) -> usize {
        self.quota_denials.load(Ordering::Relaxed)
    }

    /// Alocações pendentes (allocs - frees)
    pub fn outstanding(&self) -> usize {
        let allocs = self.alloc_count.load(Ordering::Relaxed);
        let frees = self.free_count.load(Ordering::Relaxed);
        allocs.saturating_sub(frees)
    }

    /// Verifica se há leak provável
    pub fn has_probable_leak(&self) -> bool {
        // Se alocamos muito mais do que liberamos, pode ser leak
        let allocs = self.alloc_count.load(Ordering::Relaxed);
        let frees = self.free_count.load(Ordering::Relaxed);

        // Threshold: 90% difference
        allocs > 100 && frees < allocs / 10
    }
}

// =============================================================================
// ARMAZENAMENTO GLOBAL
// =============================================================================

/// Número máximo de subsistemas
const MAX_SUBSYSTEMS: usize = 256;

/// Estatísticas globais por subsistema
static STATS: [SubsystemStats; MAX_SUBSYSTEMS] = {
    const EMPTY: SubsystemStats = SubsystemStats::new();
    [EMPTY; MAX_SUBSYSTEMS]
};

/// Flag de inicialização
static INITIALIZED: AtomicUsize = AtomicUsize::new(0);

/// Inicializa o sistema de accounting
pub fn init() {
    if INITIALIZED
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        // Aplicar quotas padrão
        for subsys in Subsystem::all() {
            let quota = subsys.default_quota();
            if quota > 0 {
                STATS[*subsys as usize].set_quota(quota);
            }
        }

        crate::kinfo!("(Accounting) Inicializado com quotas padrão");
    }
}

/// Obtém estatísticas de um subsistema
pub fn get_stats(subsys: Subsystem) -> &'static SubsystemStats {
    &STATS[subsys as u8 as usize]
}

// =============================================================================
// RELATÓRIOS
// =============================================================================

/// Imprime relatório de uso de memória
pub fn print_memory_report() {
    crate::kinfo!("╔═══════════════════════════════════════════════════════════════╗");
    crate::kinfo!("║              RELATÓRIO DE MEMÓRIA POR SUBSISTEMA              ║");
    crate::kinfo!("╠════════════════╦════════════╦════════════╦════════════╦═══════╣");
    crate::kinfo!("║   Subsistema   ║   Alocado  ║    Pico    ║   Quota    ║ Leaks?║");
    crate::kinfo!("╠════════════════╬════════════╬════════════╬════════════╬═══════╣");

    let mut total_allocated = 0usize;
    let mut total_peak = 0usize;

    for subsys in Subsystem::all() {
        let stats = get_stats(*subsys);
        let allocated = stats.allocated_bytes();
        let peak = stats.peak_bytes();
        let quota = stats.quota_bytes();
        let has_leak = stats.has_probable_leak();

        if stats.allocation_count() > 0 {
            total_allocated += allocated;
            total_peak = total_peak.max(peak);

            let quota_str = if quota > 0 {
                format_bytes(quota)
            } else {
                "∞".into()
            };

            let leak_str = if has_leak { "⚠" } else { " " };

            crate::kinfo!(
                "║ {:14} ║ {:>10} ║ {:>10} ║ {:>10} ║   {}   ║",
                subsys.name(),
                format_bytes(allocated),
                format_bytes(peak),
                quota_str,
                leak_str
            );
        }
    }

    crate::kinfo!("╠════════════════╬════════════╬════════════╬════════════╬═══════╣");
    crate::kinfo!(
        "║     TOTAL      ║ {:>10} ║ {:>10} ║     -      ║       ║",
        format_bytes(total_allocated),
        format_bytes(total_peak)
    );
    crate::kinfo!("╚════════════════╩════════════╩════════════╩════════════╩═══════╝");
}

/// Imprime resumo curto
pub fn print_summary() {
    let mut total = 0usize;
    let mut with_usage = 0usize;

    for subsys in Subsystem::all() {
        let stats = get_stats(*subsys);
        if stats.allocation_count() > 0 {
            total += stats.allocated_bytes();
            with_usage += 1;
        }
    }

    crate::kinfo!(
        "(Accounting) {} subsistemas ativos, {} bytes alocados",
        with_usage,
        format_bytes(total)
    );
}

/// Formata bytes para exibição legível
fn format_bytes(bytes: usize) -> alloc::string::String {
    use alloc::string::ToString;

    if bytes >= 1024 * 1024 * 1024 {
        let gb = bytes / (1024 * 1024 * 1024);
        let mb = (bytes % (1024 * 1024 * 1024)) / (1024 * 1024);
        alloc::format!("{}.{}GB", gb, mb / 100)
    } else if bytes >= 1024 * 1024 {
        let mb = bytes / (1024 * 1024);
        let kb = (bytes % (1024 * 1024)) / 1024;
        alloc::format!("{}.{}MB", mb, kb / 100)
    } else if bytes >= 1024 {
        let kb = bytes / 1024;
        alloc::format!("{}KB", kb)
    } else {
        alloc::format!("{}B", bytes)
    }
}

// =============================================================================
// DIAGNÓSTICO
// =============================================================================

/// Verifica todos os subsistemas por leaks
pub fn check_for_leaks() {
    let mut found_leaks = false;

    for subsys in Subsystem::all() {
        let stats = get_stats(*subsys);
        if stats.has_probable_leak() {
            crate::kwarn!(
                "(Accounting) Possível leak em {}: {} allocs, {} frees, {} bytes",
                subsys.name(),
                stats.allocation_count(),
                stats.free_count(),
                stats.allocated_bytes()
            );
            found_leaks = true;
        }
    }

    if !found_leaks {
        crate::kinfo!("(Accounting) Nenhum leak detectado");
    }
}

/// Reseta contadores (para testes)
#[cfg(debug_assertions)]
pub fn reset_all() {
    for i in 0..MAX_SUBSYSTEMS {
        STATS[i].allocated.store(0, Ordering::Relaxed);
        STATS[i].alloc_count.store(0, Ordering::Relaxed);
        STATS[i].free_count.store(0, Ordering::Relaxed);
        STATS[i].peak.store(0, Ordering::Relaxed);
        STATS[i].quota_denials.store(0, Ordering::Relaxed);
    }
}
