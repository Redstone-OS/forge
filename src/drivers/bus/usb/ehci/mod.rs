//! # EHCI - Enhanced Host Controller Interface (USB 2.0)
//!
//! Este módulo implementa o driver do **EHCI**, o host controller
//! para USB 2.0 (High Speed - 480 Mbps).
//!
//! ## Características:
//! - Suporta USB 2.0 High Speed (480 Mbps)
//! - Mais simples que xHCI
//! - Legado - usado como fallback quando xHCI não disponível
//!
//! ## STUB:
//! Implementação mínima. xHCI é preferido.

// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::bus::pci;
use crate::sync::Spinlock;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa controllers EHCI.
pub fn init() {
    crate::kinfo!("(EHCI) Inicializando controllers EHCI...");

    let host_controllers = super::HOST_CONTROLLERS.lock();
    let mut count = 0;

    for hc in host_controllers.iter() {
        if hc.controller_type != super::HostControllerType::Ehci {
            continue;
        }

        crate::kinfo!("(EHCI) Controller encontrado em", hc.mmio_base);
        count += 1;

        // TODO: Inicializar controller
        crate::kwarn!("(EHCI) Inicialização não implementada - usando xHCI");
    }

    *INITIALIZED.lock() = count > 0;

    if count > 0 {
        crate::kinfo!(
            "(EHCI) Encontrados",
            count,
            "controllers (não inicializados)"
        );
    }
}

/// Desliga controllers EHCI.
pub fn shutdown() {
    crate::kinfo!("(EHCI) Shutdown");
}

/// Verifica se EHCI está disponível.
pub fn is_available() -> bool {
    *INITIALIZED.lock()
}
