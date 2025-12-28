//! High Resolution Timer (HRTimer)
//!
//! Propósito: Temporizadores de Alta Resolução.
//! Diferente dos timers baseados em Jiffies, estes usam nanosegundos e dependem
//! de fontes de relógio precisas (como TSC ou HPET) abstraídas pelo hardware.
//!
//! Detalhes de Implementação:
//! - Baseado em tempo absoluto em nanosegundos.
//! - Ideal para agendamento preciso de mídia ou controle em tempo real.

use alloc::boxed::Box;

/// Callback para HRTimer
pub trait HrTimerCallback: Send + Sync {
    fn on_expiration(&mut self);
}

/// Timer de Alta Resolução
pub struct HrTimer {
    /// Momento de expiração em nanosegundos (tempo monotônico)
    pub expires_ns: u64,
    /// Callback
    pub callback: Box<dyn FnMut() + Send + Sync>,
}

impl HrTimer {
    /// Cria novo HRTimer
    pub fn new<F>(expires_ns: u64, callback: F) -> Self
    where
        F: FnMut() + Send + Sync + 'static,
    {
        Self {
            expires_ns,
            callback: Box::new(callback),
        }
    }

    /// Verifica se expirou dado o tempo atual em ns
    pub fn is_expired(&self, current_ns: u64) -> bool {
        current_ns >= self.expires_ns
    }

    /// Executa o callback
    pub fn run(&mut self) {
        (self.callback)();
    }
}
