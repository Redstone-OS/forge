//! # Module Watchdog
//!
//! Monitora a saúde dos módulos e detecta falhas.
//!
//! ## Responsabilidades
//! - Verificar healthcheck periodicamente
//! - Detectar módulos travados (timeout)
//! - Reportar falhas ao supervisor

use super::ModuleId;
use alloc::collections::BTreeMap;

/// Status de saúde de um módulo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Módulo está saudável
    Healthy,
    /// Módulo demorou para responder
    Slow,
    /// Módulo não respondeu ao healthcheck
    Unresponsive,
    /// Módulo reportou erro
    Error,
    /// Módulo está morto (crash)
    Dead,
}

/// Registro de um módulo no watchdog
struct WatchedModule {
    /// Último healthcheck bem-sucedido (em ticks)
    last_healthy: u64,
    /// Contagem de falhas consecutivas
    consecutive_failures: u32,
    /// Status atual
    status: HealthStatus,
}

/// Watchdog de módulos
pub struct ModuleWatchdog {
    /// Módulos monitorados
    watched: BTreeMap<ModuleId, WatchedModule>,
    /// Intervalo de healthcheck em ticks
    check_interval: u64,
    /// Timeout em ticks
    timeout: u64,
    /// Máximo de falhas antes de declarar morto
    max_failures: u32,
    /// Se watchdog está ativo
    active: bool,
}

impl ModuleWatchdog {
    /// Cria um novo watchdog
    pub const fn new() -> Self {
        Self {
            watched: BTreeMap::new(),
            check_interval: 100, // ~1 segundo a 100Hz
            timeout: 500,        // ~5 segundos
            max_failures: 3,
            active: false,
        }
    }

    /// Inicializa o watchdog
    pub fn init(&mut self) {
        self.active = true;
        crate::kdebug!("(Watchdog) Inicializado");
    }

    /// Registra um módulo para monitoramento
    pub fn register(&mut self, id: ModuleId) {
        let now = crate::drivers::timer::ticks();

        self.watched.insert(
            id,
            WatchedModule {
                last_healthy: now,
                consecutive_failures: 0,
                status: HealthStatus::Healthy,
            },
        );

        crate::ktrace!("(Watchdog) Módulo registrado: ", id.as_u64());
    }

    /// Remove um módulo do monitoramento
    pub fn unregister(&mut self, id: ModuleId) {
        self.watched.remove(&id);
        crate::ktrace!("(Watchdog) Módulo removido: ", id.as_u64());
    }

    /// Atualiza status de um módulo (chamado pelo módulo)
    pub fn heartbeat(&mut self, id: ModuleId) {
        if let Some(module) = self.watched.get_mut(&id) {
            let now = crate::drivers::timer::ticks();
            module.last_healthy = now;
            module.consecutive_failures = 0;
            module.status = HealthStatus::Healthy;
        }
    }

    /// Reporta erro de um módulo
    pub fn report_error(&mut self, id: ModuleId) {
        if let Some(module) = self.watched.get_mut(&id) {
            module.status = HealthStatus::Error;
            module.consecutive_failures += 1;
        }
    }

    /// Verifica todos os módulos (chamado periodicamente pelo timer)
    pub fn check_all(&mut self) -> alloc::vec::Vec<(ModuleId, HealthStatus)> {
        if !self.active {
            return alloc::vec::Vec::new();
        }

        let now = crate::drivers::timer::ticks();
        let mut problems = alloc::vec::Vec::new();

        for (&id, module) in self.watched.iter_mut() {
            let elapsed = now.saturating_sub(module.last_healthy);

            if elapsed > self.timeout {
                module.consecutive_failures += 1;

                if module.consecutive_failures >= self.max_failures {
                    module.status = HealthStatus::Dead;
                } else {
                    module.status = HealthStatus::Unresponsive;
                }

                problems.push((id, module.status));
            } else if elapsed > self.check_interval * 2 {
                module.status = HealthStatus::Slow;
            }
        }

        problems
    }

    /// Obtém status de um módulo
    pub fn get_status(&self, id: ModuleId) -> Option<HealthStatus> {
        self.watched.get(&id).map(|m| m.status)
    }

    /// Lista módulos com problemas
    pub fn list_problems(&self) -> alloc::vec::Vec<ModuleId> {
        self.watched
            .iter()
            .filter(|(_, m)| m.status != HealthStatus::Healthy)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Configura intervalo de healthcheck
    pub fn set_check_interval(&mut self, ticks: u64) {
        self.check_interval = ticks;
    }

    /// Configura timeout
    pub fn set_timeout(&mut self, ticks: u64) {
        self.timeout = ticks;
    }
}
