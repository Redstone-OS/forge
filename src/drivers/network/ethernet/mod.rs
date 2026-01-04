//! # Ethernet Drivers
//!
//! Este módulo contém drivers para placas de rede Ethernet físicas e virtuais.
//!
//! ## Drivers Planejados:
//!
//! ### Prioridade Alta:
//! - **e1000**: Intel Gigabit Ethernet
//!   - Muito comum em VMs (VMware, VirtualBox, QEMU)
//!   - Documentação pública disponível
//!
//! ### Prioridade Média:
//! - **e1000e**: Intel Gigabit Ethernet (moderno)
//! - **igb**: Intel 82575/82576 Gigabit
//!
//! ### Prioridade Baixa:
//! - **RTL8139**: Realtek Fast Ethernet (legado)
//! - **RTL8169**: Realtek Gigabit
//!
//! ## STUB:
//! Todos os drivers são stubs por enquanto.

pub mod e1000; // Intel e1000 (futuro)
pub mod rtl8139; // Realtek RTL8139 (futuro)

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa drivers Ethernet.
pub fn init() {
    crate::kinfo!("(Ethernet) Inicializando drivers Ethernet...");

    // TODO: Detectar placas via PCI e carregar drivers apropriados

    crate::kwarn!("(Ethernet) Drivers não implementados");
}

/// Desliga drivers Ethernet.
pub fn shutdown() {
    crate::kinfo!("(Ethernet) Shutdown");
}
