//! # kswapd - Kernel Swap Daemon
//!
//! Thread que roda em background recuperando memória.

use core::sync::atomic::{AtomicBool, Ordering};

/// kswapd está rodando?
static KSWAPD_RUNNING: AtomicBool = AtomicBool::new(false);

/// Inicia o kswapd
pub fn start_kswapd() {
    if KSWAPD_RUNNING.load(Ordering::Acquire) {
        return; // Já está rodando
    }

    // TODO: Criar kernel thread para kswapd
    // Por enquanto, apenas marca como "rodando"
    KSWAPD_RUNNING.store(true, Ordering::Release);

    crate::kinfo!("(KSWAPD) Started");
}

/// Para o kswapd
pub fn stop_kswapd() {
    KSWAPD_RUNNING.store(false, Ordering::Release);
}

/// Acorda o kswapd (chamado quando memória está baixa)
pub fn wake_up() {
    if !KSWAPD_RUNNING.load(Ordering::Acquire) {
        return;
    }

    // TODO: Sinalizar thread para acordar
    kswapd_tick();
}

/// Tick do kswapd (um ciclo de trabalho)
fn kswapd_tick() {
    let pressure = super::get_pressure();

    match pressure {
        super::MemoryPressure::None => {
            // Nada a fazer, dormir
        }
        super::MemoryPressure::Low => {
            // Recuperar algumas páginas cautelosamente
            super::evict_pages(16);
        }
        super::MemoryPressure::Medium => {
            // Recuperar mais agressivamente
            super::evict_pages(64);
        }
        super::MemoryPressure::Critical => {
            // Recuperação de emergência
            super::evict_pages(256);

            // Se ainda crítico, considerar OOM
            if super::get_pressure() == super::MemoryPressure::Critical {
                crate::kwarn!("(KSWAPD) Still critical after eviction, may need OOM");
            }
        }
    }
}
