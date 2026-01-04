//! # xHCI - eXtensible Host Controller Interface (USB 3.0+)
//!
//! Este módulo implementa o driver do **xHCI**, o host controller padrão
//! para USB 3.0 e superior. É o controller mais complexo mas também
//! mais capaz.
//!
//! ## Características:
//! - Suporta todas as velocidades USB (Low, Full, High, Super, Super+)
//! - Command Ring para enviar comandos ao controller
//! - Event Ring para receber respostas
//! - Transfer Rings por endpoint
//! - Slots para cada dispositivo
//!
//! ## Estruturas Principais:
//! - **DCBAA**: Device Context Base Address Array
//! - **Command Ring**: Fila de comandos
//! - **Event Ring**: Fila de eventos/completions
//! - **Transfer Ring**: Fila de transfers por endpoint
//!
//! ## STUB:
//! Estruturas básicas definidas. Implementação real incompleta.

pub mod controller; // Controller principal
pub mod device; // Device slots
pub mod port; // Gerenciamento de portas
pub mod regs; // Registradores xHCI
pub mod ring; // Transfer/Command/Event rings
pub mod structs; // TRBs e contextos
pub mod transfer; // Operações de transfer
pub mod types; // Tipos e constantes

// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::bus::pci;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// Re-exports
pub use controller::XhciController;
pub use types::*;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de controllers xHCI.
static XHCI_CONTROLLERS: Spinlock<Vec<Arc<XhciController>>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa todos os controllers xHCI detectados.
pub fn init() {
    crate::kinfo!("(xHCI) Inicializando controllers xHCI...");

    let host_controllers = super::HOST_CONTROLLERS.lock();

    for hc in host_controllers.iter() {
        if hc.controller_type != super::HostControllerType::Xhci {
            continue;
        }

        crate::kinfo!("(xHCI) Inicializando controller em", hc.mmio_base);

        match XhciController::new(hc.mmio_base, hc.irq) {
            Some(controller) => {
                let arc = Arc::new(controller);
                XHCI_CONTROLLERS.lock().push(arc);
            }
            None => {
                crate::kerror!("(xHCI) Falha ao inicializar controller");
            }
        }
    }

    let count = XHCI_CONTROLLERS.lock().len();
    *INITIALIZED.lock() = count > 0;

    crate::kinfo!("(xHCI) Inicializados", count, "controllers");
}

/// Desliga todos os controllers xHCI.
pub fn shutdown() {
    crate::kinfo!("(xHCI) Shutdown...");

    let controllers = XHCI_CONTROLLERS.lock();
    for ctrl in controllers.iter() {
        ctrl.shutdown();
    }
}

/// Retorna número de controllers xHCI.
pub fn controller_count() -> usize {
    XHCI_CONTROLLERS.lock().len()
}

/// Retorna referência ao primeiro controller.
pub fn get_primary_controller() -> Option<Arc<XhciController>> {
    XHCI_CONTROLLERS.lock().first().cloned()
}
