//! # OOM Killer
//!
//! Mata processos quando não há mais memória.

use core::sync::atomic::{AtomicU64, Ordering};

/// Processos mortos por OOM
pub static OOM_KILLS: AtomicU64 = AtomicU64::new(0);

/// Seleciona e mata um processo para liberar memória
pub fn oom_kill() -> bool {
    crate::kerror!("(OOM) Out of memory! Selecting victim...");

    // TODO: Implementar seleção de vítima
    // Critérios:
    // - Maior uso de memória
    // - Menor tempo de CPU (menos importante)
    // - Não é processo crítico do sistema

    let victim = select_victim();

    if let Some(pid) = victim {
        crate::kerror!("(OOM) Killing process {}", pid);
        // TODO: Enviar SIGKILL para o processo
        OOM_KILLS.fetch_add(1, Ordering::Relaxed);
        true
    } else {
        crate::kerror!("(OOM) No suitable victim found!");
        false
    }
}

/// Seleciona processo vítima
fn select_victim() -> Option<u64> {
    // TODO: Iterar sobre processos e calcular scores
    // Por enquanto, retorna None
    None
}

/// Calcula score OOM de um processo
/// Maior score = maior chance de ser morto
pub fn oom_score(pid: u64) -> i32 {
    // Fatores:
    // + Uso de memória (RSS)
    // + Tempo de execução
    // - Prioridade
    // - Se é processo crítico

    let _ = pid;
    0
}

/// Marca processo como imune a OOM
pub fn set_oom_immune(_pid: u64, _immune: bool) {
    // TODO: Marcar processo
}
