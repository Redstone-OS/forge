//! # Sistema de Eventos de Hardware (Event Layer)
//!
//! Este módulo implementa o padrão **Observer (Pub/Sub)** para eventos
//! de hardware. Permite que outros subsistemas reajam a mudanças:
//!
//! - **Hotplug**: Inserção/remoção de dispositivos (USB, PCI)
//! - **Estado**: Transições Ready → Failing → Dead
//! - **Energia**: Suspensão e retomada de dispositivos
//! - **Erros**: Falhas detectadas pelo monitor de saúde
//!
//! ## Uso Típico:
//! - VFS assina eventos de Storage para montar/desmontar automaticamente
//! - GUI assina eventos de Display para reconfigurar monitores
//! - PowerManager assina eventos de energia para coordenar suspensão
//!
//! ## Filosofia:
//! Eventos são entregues de forma **síncrona** e **rápida**.
//! Handlers não devem bloquear - se precisar de processamento pesado,
//! devem enfileirar trabalho para uma thread worker.

use super::device::{DeviceId, DeviceState};
use super::driver::DriverError;
use super::power::PowerState;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE EVENTO
// =============================================================================

/// Eventos de hardware suportados pelo RDS.
#[derive(Debug, Clone, Copy)]
pub enum DeviceEvent {
    /// Novo dispositivo foi detectado e registrado.
    /// Emitido após probe() bem-sucedido.
    Added(DeviceId),

    /// Dispositivo foi removido (hotplug) ou desativado.
    /// Emitido após remove() ou isolamento.
    Removed(DeviceId),

    /// Estado do dispositivo mudou.
    /// Ex: Initializing → Ready, Ready → Failing.
    StateChanged(DeviceId, DeviceState),

    /// Estado de energia mudou.
    /// Ex: D0 → D3 (suspensão).
    PowerChange(DeviceId, PowerState),

    /// Erro crítico detectado pelo monitor de saúde.
    /// Emitido antes de acionar recuperação.
    ErrorDetected(DeviceId, DriverError),

    /// Driver foi recarregado (hot-reload).
    DriverReloaded(DeviceId),

    /// Interrupção de hardware recebida.
    /// Usado no slow path de IRQ.
    Interrupt(DeviceId, u8),
}

impl DeviceEvent {
    /// Retorna o ID do dispositivo associado ao evento.
    pub fn device_id(&self) -> DeviceId {
        match self {
            Self::Added(id) => *id,
            Self::Removed(id) => *id,
            Self::StateChanged(id, _) => *id,
            Self::PowerChange(id, _) => *id,
            Self::ErrorDetected(id, _) => *id,
            Self::DriverReloaded(id) => *id,
            Self::Interrupt(id, _) => *id,
        }
    }

    /// Retorna nome legível do tipo de evento.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added(_) => "Added",
            Self::Removed(_) => "Removed",
            Self::StateChanged(_, _) => "StateChanged",
            Self::PowerChange(_, _) => "PowerChange",
            Self::ErrorDetected(_, _) => "ErrorDetected",
            Self::DriverReloaded(_) => "DriverReloaded",
            Self::Interrupt(_, _) => "Interrupt",
        }
    }
}

// =============================================================================
// TRAIT DE LISTENER
// =============================================================================

/// Interface para ouvintes de eventos de hardware.
///
/// Implementadores recebem todos os eventos e decidem quais processar.
pub trait DeviceListener: Send + Sync {
    /// Chamado quando um evento ocorre.
    ///
    /// ## IMPORTANTE:
    /// - Não bloqueie nesta função!
    /// - Se precisar de processamento pesado, enfileire para worker thread
    /// - Mantenha o handler rápido para não atrasar outros listeners
    fn on_event(&self, event: DeviceEvent);

    /// Retorna nome do listener (para debug).
    fn name(&self) -> &'static str {
        "unknown"
    }
}

// =============================================================================
// GERENCIADOR DE EVENTOS
// =============================================================================

/// Gerenciador central de eventos.
struct EventManager {
    /// Lista de listeners registrados.
    subscribers: Vec<Box<dyn DeviceListener>>,

    /// Flag de inicialização.
    initialized: bool,

    /// Contador de eventos emitidos (para debug).
    event_count: u64,
}

/// Instância global do gerenciador de eventos.
static EVENT_MANAGER: Spinlock<EventManager> = Spinlock::new(EventManager {
    subscribers: Vec::new(),
    initialized: false,
    event_count: 0,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o sistema de eventos.
pub fn init() {
    crate::kinfo!("(Events) Inicializando sistema de eventos...");

    let mut mgr = EVENT_MANAGER.lock();
    mgr.initialized = true;

    // Registra listener de log padrão
    drop(mgr); // Libera lock antes de registrar
    subscribe(Box::new(LogListener));

    crate::kinfo!("(Events) Sistema pronto");
}

/// Emite um evento para todos os listeners.
///
/// Esta função é **síncrona** - todos os handlers são chamados
/// antes de retornar. Mantenha handlers rápidos!
pub fn emit(event: DeviceEvent) {
    let mgr = EVENT_MANAGER.lock();

    if !mgr.initialized {
        // Sistema ainda não inicializado - ignora silenciosamente
        return;
    }

    // Atualiza contador (para telemetria)
    // Note: precisamos de &mut, mas temos &, então fazemos via interior mutability
    // na próxima versão

    // Notifica todos os subscribers
    for sub in mgr.subscribers.iter() {
        sub.on_event(event);
    }
}

/// Registra um novo listener para eventos.
pub fn subscribe(listener: Box<dyn DeviceListener>) {
    let name = listener.name();
    crate::kinfo!("(Events) Novo listener:", name);
    EVENT_MANAGER.lock().subscribers.push(listener);
}

/// Remove um listener (por nome).
///
/// ## STUB:
/// Implementação completa requer identificadores únicos.
pub fn unsubscribe(name: &str) {
    crate::kwarn!("(Events) unsubscribe() não totalmente implementado");
    // TODO: Implementar remoção por ID ou referência
}

/// Retorna número de listeners registrados.
pub fn subscriber_count() -> usize {
    EVENT_MANAGER.lock().subscribers.len()
}

/// Retorna número total de eventos emitidos.
pub fn total_events() -> u64 {
    EVENT_MANAGER.lock().event_count
}

// =============================================================================
// LISTENERS PADRÃO
// =============================================================================

/// Listener que apenas loga eventos.
///
/// Registrado automaticamente durante init() para debug.
pub struct LogListener;

impl DeviceListener for LogListener {
    fn on_event(&self, event: DeviceEvent) {
        let id = event.device_id();

        match event {
            DeviceEvent::Added(_) => {
                crate::kinfo!("(Event) Dispositivo adicionado, ID:", id.0);
            }
            DeviceEvent::Removed(_) => {
                crate::kwarn!("(Event) Dispositivo removido, ID:", id.0);
            }
            DeviceEvent::StateChanged(_, new_state) => {
                crate::kinfo!(
                    "(Event) Estado alterado, ID:",
                    id.0,
                    "→",
                    new_state.as_str()
                );
            }
            DeviceEvent::PowerChange(_, new_power) => {
                crate::kinfo!("(Event) Energia alterada, ID:", id.0);
            }
            DeviceEvent::ErrorDetected(_, err) => {
                crate::kerror!("(Event) Erro detectado, ID:", id.0, "erro:", err.as_str());
            }
            DeviceEvent::DriverReloaded(_) => {
                crate::kinfo!("(Event) Driver recarregado, ID:", id.0);
            }
            DeviceEvent::Interrupt(_, irq) => {
                // IRQs são muito frequentes - não loga por padrão
            }
        }
    }

    fn name(&self) -> &'static str {
        "LogListener"
    }
}
