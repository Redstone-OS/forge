//! # Network Device Drivers Subsystem
//!
//! Este módulo gerencia os adaptadores de rede (NICs) do RedstoneOS.
//! Ele separa os drivers de hardware da pilha de protocolos (NET Stack).
//!
//! ## Estrutura:
//! - **traits**: Abstrações compartilhadas (`NetworkAdapter`).
//! - **ethernet/**: Drivers para placas físicas (Intel, Realtek).
//! - **virtio/**: Suporte a redes virtualizadas.
//! - **mii/**: Gerenciamento de interface física (PHY/Auto-negotiation).
//! - **wifi/**: Drivers para redes sem fio (*Planejado*).

pub mod ethernet;
pub mod loopback;
pub mod mii;
pub mod traits;
pub mod virtio;
pub mod wifi;

use crate::drivers::base::driver::Driver;
use alloc::sync::Arc;

/// Inicializa e registra todos os drivers de rede disponíveis no RDM.
pub fn init() {
    crate::kinfo!("(Net) Inicializando subsistema de drivers de rede...");

    // 1. Inicializa drivers Ethernet (Físicos)
    ethernet::init();

    // 2. Registra o adaptador Loopback
    crate::drivers::base::register_driver(Arc::new(loopback::LoopbackDriver) as Arc<dyn Driver>);

    // 3. Registra drivers VirtIO
    crate::drivers::base::register_driver(Arc::new(virtio::VirtioNetDriver) as Arc<dyn Driver>);

    // 4. Reservado para WiFi
    // wifi::init();

    crate::kinfo!("(Net) Subsistema de rede pronto para detecção de hardware.");
}
