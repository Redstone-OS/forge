//! # Sistema de Eventos de Hardware (Event Layer)
//!
//! O canal de comunicação para mudanças de estado "Hot" no hardware.
//! Implementa o padrão Observer (Publisher/Subscriber) para permitir que outros
//! subsistemas reajam a mudanças na topologia do hardware.
//!
//! ## Eventos Críticos:
//! - **Hotplug**: Inserção ou remoção de dispositivos (USB, PCI Hotplug).
//! - **Recuperação**: Notificações quando um driver falha ou é reiniciado.
//! - **Energia**: Alertas de bateria fraca ou mudança de perfil de consumo.
//!
//! Este sistema é o que permite que o RedstoneOS monte automaticamente um
//! pendrive quando ele é conectado ou atualize a lista de monitores.

use super::device::{DeviceId, DeviceState};
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Tipos de eventos suportados pelo modelo de dispositivos
#[derive(Debug, Clone, Copy)]
pub enum DeviceEvent {
    /// Um novo hardware foi detectado e registrado
    Added(DeviceId),
    /// Um hardware foi removido fisicamente ou desativado
    Removed(DeviceId),
    /// O estado operacional mudou (ex: de Initializing para Ready)
    StateChanged(DeviceId, DeviceState),
    /// Mudança no estado de energia (ex: Suspensão iniciada)
    PowerChange(DeviceId, super::power::PowerState),
    /// Uma erro crítico foi detectado pelo Monitor de Saúde
    ErrorDetected(DeviceId, super::driver::DriverError),
}

/// Interface para ouvintes de eventos de hardware
pub trait DeviceListener: Send + Sync {
    fn on_event(&self, event: DeviceEvent);
}

/// Gerenciador de Assinaturas de Eventos
struct EventManager {
    subscribers: Vec<Box<dyn DeviceListener>>,
}

static EVENT_MANAGER: Spinlock<EventManager> = Spinlock::new(EventManager {
    subscribers: Vec::new(),
});

/// Notifica todos os assinantes interessados sobre um evento de hardware.
/// Esta chamada deve ser rápida e não-bloqueante.
pub fn emit(event: DeviceEvent) {
    let mgr = EVENT_MANAGER.lock();
    for sub in mgr.subscribers.iter() {
        sub.on_event(event);
    }
}

/// Registra um novo ouvinte para eventos globais de hardware.
/// Útil para o VFS, Gerenciador de Janelas e Daemon de Energia.
pub fn subscribe(listener: Box<dyn DeviceListener>) {
    EVENT_MANAGER.lock().subscribers.push(listener);
}

/// Exemplo de ouvinte simples: Log System
pub struct LogListener;
impl DeviceListener for LogListener {
    fn on_event(&self, event: DeviceEvent) {
        match event {
            DeviceEvent::Added(id) => crate::kinfo!("(Event) Novo hardware detectado, ID:", id.0),
            DeviceEvent::Removed(id) => crate::kwarn!("(Event) Hardware removido, ID:", id.0),
            DeviceEvent::ErrorDetected(id, err) => {
                crate::kerror!("(Event) Erro em dispositivo:", id.0)
            }
            _ => {}
        }
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Notificações para o Userspace (uevent):
//    - Criar um canal para enviar esses eventos para processos em userspace
//      (similar ao netlink uevent do Linux). Isso permitiria que apps de
//      configuração reagissem à inserção de hardware.
//
// 2. Filtros de Eventos:
//    - Permitir que assinantes se interessem apenas por certas classes de
//      dispositivos (ex: VFS assina apenas eventos da classe Storage).
//
// 3. Sistema de Prioridade:
//    - Garantir que eventos críticos de erro sejam entregues antes de eventos
//      de adição/remoção.
//
// 4. Acúmulo e Throttling:
//    - Evitar tempestades de eventos (event storms) caso um barramento
//      interfira com muitos sinais falsos rapidamente.
//
// 5. Histórico de Eventos:
//    - Manter um buffer circular com os últimos N eventos para depuração
//      pós-morte e suporte a logs de sistema.
