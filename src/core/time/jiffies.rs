//! Arquivo: core/time/jiffies.rs
//!
//! Propósito: Jiffies (Contador de ticks do sistema).
//! "Jiffies" é o termo histórico (do Linux) para ticks de relógio.
//! Útil para timeouts grosseiros e medição de uptime.
//!
//! Detalhes de Implementação:
//! - Usa AtomicU64 para ser thread-safe e lock-free.
//! - Incrementado pelo timer interrupt handler.

//! Contador de Jiffies (Ticks)

use core::sync::atomic::{AtomicU64, Ordering};

/// Ticks desde o boot.
/// Visibilidade de crate para permitir incremento pelo timer handler.
pub(crate) static JIFFIES: AtomicU64 = AtomicU64::new(0);

/// Frequência do Tick (Ticks por segundo)
/// TODO: Tornar configurável
pub const HZ: u64 = 100;

/// Retorna o número atual de jiffies.
#[inline]
pub fn get_jiffies() -> u64 {
    JIFFIES.load(Ordering::Relaxed)
}

/// Incrementa o contador de jiffies.
/// Deve ser chamado APENAS pelo handler de interrupção do timer.
#[inline]
pub fn inc_jiffies() {
    JIFFIES.fetch_add(1, Ordering::Relaxed);
}

/// Converte segundos para jiffies.
#[inline]
pub const fn seconds_to_jiffies(seconds: u64) -> u64 {
    seconds * HZ
}

/// Converte milisegundos para jiffies.
#[inline]
pub const fn millis_to_jiffies(millis: u64) -> u64 {
    (millis * HZ) / 1000
}
