//! # Timer Subsystem
//!
//! Gerencia os temporizadores do sistema (PIT, HPET, TSC).
//! Fornece abstração unificada para timing no kernel.
//!
//! ## Hierarquia de Fallback:
//! 1. **HPET** - Alta precisão (preferido)
//! 2. **TSC** - Ciclos de CPU (mais rápido, mas precisa calibração)
//! 3. **PIT** - Legacy 8254 (sempre disponível)

pub mod hpet;
pub mod pit;
pub mod tsc;

use crate::drivers::base::driver::Driver;
use crate::drivers::system::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

// Re-export PIT init para uso direto no boot
pub use pit::init as init_pit;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Timer ativo atualmente.
static ACTIVE_TIMER: Spinlock<Option<TimerSource>> = Spinlock::new(None);

/// Frequência do timer em Hz.
static TIMER_FREQ: Spinlock<u64> = Spinlock::new(250);

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de timer.
pub fn init() {
    crate::kinfo!("(Timer) Inicializando subsistema de timer...");

    // Registra drivers no RDM
    crate::drivers::base::register_driver(Arc::new(hpet::HpetDriver) as Arc<dyn Driver>);

    // Por enquanto usamos PIT como padrão
    *ACTIVE_TIMER.lock() = Some(TimerSource::Pit);

    crate::kinfo!("(Timer) Subsistema inicializado (fonte: PIT)");
}

/// Define o timer ativo.
pub fn set_active_timer(source: TimerSource) {
    *ACTIVE_TIMER.lock() = Some(source);
    crate::kinfo!("(Timer) Fonte ativa:", source);
}

/// Retorna o timer ativo.
pub fn get_active_timer() -> Option<TimerSource> {
    *ACTIVE_TIMER.lock()
}

// =============================================================================
// FUNÇÕES DE TEMPO
// =============================================================================

/// Retorna ticks atuais do sistema (monotônico).
pub fn ticks() -> u64 {
    // Usa jiffies do kernel (incrementado pelo PIT IRQ)
    crate::core::time::jiffies::get_jiffies()
}

/// Retorna frequência do timer base em Hz.
pub fn frequency() -> u64 {
    *TIMER_FREQ.lock()
}

/// Define frequência do timer.
pub fn set_frequency(freq: u64) {
    *TIMER_FREQ.lock() = freq;
}

/// Converte ticks para milissegundos.
pub fn ticks_to_ms(ticks: u64) -> u64 {
    let freq = frequency();
    if freq == 0 {
        return 0;
    }
    (ticks * 1000) / freq
}

/// Converte milissegundos para ticks.
pub fn ms_to_ticks(ms: u64) -> u64 {
    (ms * frequency()) / 1000
}

/// Retorna uptime do sistema em milissegundos.
pub fn uptime_ms() -> u64 {
    ticks_to_ms(ticks())
}

/// Retorna uptime do sistema em segundos.
pub fn uptime_secs() -> u64 {
    uptime_ms() / 1000
}

// =============================================================================
// FUNÇÕES DE DELAY
// =============================================================================

/// Delay em milissegundos com multitarefa cooperativa.
///
/// Ao invés de fazer busy-wait, cede a CPU via yield_now() permitindo
/// que outros processos executem enquanto este espera.
pub fn delay_ms(ms: u64) {
    if ms == 0 {
        return;
    }

    let start = ticks();
    let target_ticks = ms_to_ticks(ms);

    while ticks() - start < target_ticks {
        // Ceder CPU para outros processos
        crate::sched::yield_now();
    }
}

/// Delay em microsegundos (busy-wait).
///
/// Para delays muito curtos onde yield não faz sentido.
/// NOTA: Precisão depende do timer disponível.
pub fn delay_us(us: u64) {
    if us == 0 {
        return;
    }

    // Para delays curtos, usa busy-wait com TSC se disponível
    let start = tsc::read_tsc();
    let _cycles = us * 1000; // Aproximação, precisa calibração

    // Fallback: converter para ms e usar delay_ms
    if us >= 1000 {
        delay_ms(us / 1000);
    } else {
        // Busy-wait com nop
        for _ in 0..us * 10 {
            core::hint::spin_loop();
        }
    }
}

/// Espera até uma condição ser verdadeira ou timeout.
///
/// Retorna true se a condição foi satisfeita, false se timeout.
pub fn wait_until<F>(timeout_ms: u64, condition: F) -> bool
where
    F: Fn() -> bool,
{
    let start = ticks();
    let timeout_ticks = ms_to_ticks(timeout_ms);

    while ticks() - start < timeout_ticks {
        if condition() {
            return true;
        }
        crate::sched::yield_now();
    }

    false
}
