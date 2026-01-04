//! # ISA Bus Driver
//!
//! Este módulo implementa o suporte ao **ISA (Industry Standard Architecture)**
//! bus - o barramento legado de PCs x86.
//!
//! ## Histórico:
//! ISA foi o barramento principal de PCs IBM AT (1984). Hoje é obsoleto,
//! mas muitos dispositivos legados ainda usam suas convenções:
//! - Portas I/O fixas
//! - IRQs fixas
//! - DMA de 8 bits
//!
//! ## Dispositivos ISA Típicos:
//! - Floppy controller (0x3F0-0x3F7)
//! - Parallel port (0x378-0x37F)
//! - Game port (0x200-0x207)
//!
//! ## Integração:
//! Na prática, a maioria dos "dispositivos ISA" são tratados como
//! Platform devices. Este módulo existe para compatibilidade.

use crate::drivers::base::bus::{Bus, BusType};
use crate::drivers::base::device::Device;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// IMPLEMENTAÇÃO DO BUS TRAIT
// =============================================================================

/// Implementação do barramento ISA.
pub struct IsaBus;

impl Bus for IsaBus {
    fn name(&self) -> &'static str {
        "ISA Legacy Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Isa
    }

    fn scan(&self) -> Vec<Device> {
        // ISA não tem scan dinâmico - dispositivos são conhecidos
        Vec::new()
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // ISA não suporta reset via bus
        false
    }

    fn shutdown(&self) {
        crate::kinfo!("(ISA) Shutdown");
    }
}

/// Retorna referência ao barramento ISA.
pub fn get_bus() -> Arc<dyn Bus> {
    Arc::new(IsaBus)
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o ISA Bus.
pub fn init() {
    crate::kinfo!("(ISA) Inicializando ISA Bus (legacy)...");

    *INITIALIZED.lock() = true;

    // Registra na base
    crate::drivers::base::bus::register(Arc::new(IsaBus));

    crate::kinfo!("(ISA) Bus inicializado");
}
