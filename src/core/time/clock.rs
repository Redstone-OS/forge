/// Arquivo: core/time/clock.rs
///
/// Propósito: Manter o tempo real (Wall Clock Time).
/// Diferente do tempo monotônico (Jiffies/Boot Time), este relógio reflete
/// a data e hora humana (UTC).
///
/// Detalhes de Implementação:
/// - Armazena segundos desde Epoch (Unix Time).
/// - Suporta ajuste de relógio (NTP no futuro).
/// - Sincroniza com RTC no boot.

//! Relógio de Tempo Real (Wall Clock)

use core::sync::atomic::{AtomicU64, Ordering};

/// Segundos e Nanosegundos desde Epoch (1970-01-01 00:00:00 UTC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSpec {
    pub seconds: u64,
    pub nanos: u32,
}

impl TimeSpec {
    pub fn new(seconds: u64, nanos: u32) -> Self {
        Self { seconds, nanos }
    }
}

pub struct SystemClock {
    // Base de tempo definida no boot (lida do RTC)
    boot_time_seconds: AtomicU64,
}

impl SystemClock {
    const fn new() -> Self {
        Self {
            boot_time_seconds: AtomicU64::new(0),
        }
    }

    /// Define o tempo de boot (chamado pelo driver RTC na inicialização)
    pub fn set_boot_time(&self, seconds: u64) {
        self.boot_time_seconds.store(seconds, Ordering::Relaxed);
    }

    /// Retorna o tempo atual aproximado (Base + Uptime)
    /// Para precisão de nanosegundos, precisaríamos somar (Jiffies * NS_PER_TICK) ou ler TSC.
    pub fn now(&self) -> TimeSpec {
        let base = self.boot_time_seconds.load(Ordering::Relaxed);
        let uptime_jiffies = super::jiffies::get_jiffies();
        
        // Conversão simples Jiffies -> Segundos
        // Assumindo HZ=100 (10ms por tick)
        let uptime_seconds = uptime_jiffies / super::jiffies::HZ;
        let remaining_ticks = uptime_jiffies % super::jiffies::HZ;
        let nanos = (remaining_ticks * (1_000_000_000 / super::jiffies::HZ)) as u32;

        TimeSpec {
            seconds: base + uptime_seconds,
            nanos,
        }
    }
}

/// Instância global do relógio de sistema
pub static WALL_CLOCK: SystemClock = SystemClock::new();
