/// Arquivo: core/time/timer.rs
///
/// Propósito: Interface genérica para temporizadores (Timers).
/// Permite agendar callbacks para serem executados após um certo intervalo de tempo.
///
/// Detalhes de Implementação:
/// - Baseado em "Jiffies" (ticks do sistema) ou nanosegundos (se HRTimer).
/// - Abstração básica para uso por drivers e subsistemas.
// Interface de Timers
use alloc::boxed::Box;

/// Callback para quando o timer expirar
pub trait TimerCallback: Send + Sync {
    fn on_expiration(&mut self);
}

/// Um timer genérico
pub struct Timer {
    /// Momento de expiração (em ticks/jiffies absolutos)
    pub expires: u64,
    /// Callback a ser executado
    pub callback: Box<dyn FnMut() + Send + Sync>,
}

impl Timer {
    /// Cria um novo timer
    pub fn new<F>(expires: u64, callback: F) -> Self
    where
        F: FnMut() + Send + Sync + 'static,
    {
        Self {
            expires,
            callback: Box::new(callback),
        }
    }

    /// Verifica se o timer expirou dado o tick atual
    pub fn is_expired(&self, current_tick: u64) -> bool {
        current_tick >= self.expires
    }

    /// Executa o callback do timer
    pub fn run(&mut self) {
        (self.callback)();
    }
}

// TODO: Implementar Timers List ou Timer Wheel para gerenciamento eficiente
// de múltiplos timers ativos.
