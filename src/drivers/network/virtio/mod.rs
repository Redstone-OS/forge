//! # VirtIO Network Driver
//!
//! Driver para **VirtIO-Net** - o dispositivo de rede paravirtualizado
//! usado por QEMU, KVM e outros hypervisors.
//!
//! ## Características:
//! - Alta performance (evita emulação de hardware real)
//! - Suporta checksum offload
//! - Suporta GSO/GRO
//! - Múltiplas queues (multi-queue)
//!
//! ## Arquitetura VirtIO-Net:
//! ```text
//! +------------------+
//! | Driver (guest)   |
//! +--------+---------+
//!          |
//! +--------v---------+
//! | VirtQueues       |
//! | - RX Queue       |
//! | - TX Queue       |
//! | - Control Queue  |
//! +--------+---------+
//!          |
//! +--------v---------+
//! | VirtIO Transport |
//! | (PCI or MMIO)    |
//! +--------+---------+
//!          |
//! +--------v---------+
//! | Hypervisor       |
//! +------------------+
//! ```
//!
//! ## STUB:
//! Estrutura definida. Implementação pendente após VirtIO bus.

use super::traits::*;
use super::NetworkStats;
use crate::drivers::bus::virtio::{VirtioDevice, VirtioDeviceType};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Tamanho do header VirtIO-Net.
pub const VIRTIO_NET_HDR_SIZE: usize = 12;

/// Tamanho máximo de pacote.
pub const VIRTIO_NET_MAX_PACKET: usize = 1514;

// =============================================================================
// ESTRUTURA DO DRIVER
// =============================================================================

/// Driver VirtIO-Net.
pub struct VirtioNetDevice {
    /// Nome do dispositivo.
    name: &'static str,

    /// Endereço MAC.
    mac: MacAddress,

    /// MTU.
    mtu: Spinlock<u16>,

    /// Estatísticas.
    stats: Spinlock<NetworkStats>,

    /// Dispositivo está habilitado?
    enabled: Spinlock<bool>,

    /// Link está ativo?
    link_up: Spinlock<bool>,
}

impl VirtioNetDevice {
    /// Cria novo driver VirtIO-Net.
    ///
    /// ## STUB:
    /// Não inicializa hardware real.
    pub fn new(name: &'static str, mac: MacAddress) -> Self {
        Self {
            name,
            mac,
            mtu: Spinlock::new(1500),
            stats: Spinlock::new(NetworkStats::default()),
            enabled: Spinlock::new(false),
            link_up: Spinlock::new(false),
        }
    }

    /// Probe de dispositivo VirtIO.
    ///
    /// ## STUB:
    /// Não implementado.
    pub fn probe(_virtio_dev: &VirtioDevice) -> Option<Self> {
        crate::kwarn!("(VirtIO-Net) probe() não implementado");
        None
    }
}

impl NetworkDevice for VirtioNetDevice {
    fn name(&self) -> &str {
        self.name
    }

    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn mtu(&self) -> u16 {
        *self.mtu.lock()
    }

    fn set_mtu(&self, mtu: u16) -> bool {
        *self.mtu.lock() = mtu;
        true
    }

    fn is_link_up(&self) -> bool {
        *self.link_up.lock()
    }

    fn link_state(&self) -> LinkState {
        if *self.link_up.lock() {
            LinkState::Up
        } else {
            LinkState::Down
        }
    }

    fn link_speed(&self) -> LinkSpeed {
        LinkSpeed::Speed10000 // VirtIO é muito rápido
    }

    fn get_stats(&self) -> NetworkStats {
        self.stats.lock().clone()
    }

    fn reset_stats(&self) {
        *self.stats.lock() = NetworkStats::default();
    }

    fn transmit(&self, _data: &[u8]) -> Result<usize, NetworkError> {
        crate::kwarn!("(VirtIO-Net) transmit() não implementado");
        Err(NetworkError::NotSupported)
    }

    fn receive(&self, _buffer: &mut [u8]) -> Result<Option<usize>, NetworkError> {
        crate::kwarn!("(VirtIO-Net) receive() não implementado");
        Err(NetworkError::NotSupported)
    }

    fn enable(&self) -> bool {
        crate::kwarn!("(VirtIO-Net) enable() não implementado");
        false
    }

    fn disable(&self) {
        *self.enabled.lock() = false;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock()
    }

    fn set_promiscuous(&self, _enabled: bool) {
        crate::kwarn!("(VirtIO-Net) set_promiscuous() não implementado");
    }

    fn add_multicast(&self, _mac: MacAddress) {
        crate::kwarn!("(VirtIO-Net) add_multicast() não implementado");
    }

    fn remove_multicast(&self, _mac: MacAddress) {
        crate::kwarn!("(VirtIO-Net) remove_multicast() não implementado");
    }
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static DEVICES: Spinlock<Vec<Arc<VirtioNetDevice>>> = Spinlock::new(Vec::new());

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa driver VirtIO-Net.
pub fn init() {
    crate::kinfo!("(VirtIO-Net) Inicializando driver...");

    // Busca dispositivos VirtIO-Net
    // TODO: Revisar no futuro
    #[allow(unused)]
    if let Some(vdev) = crate::drivers::bus::virtio::find_by_type(VirtioDeviceType::Network) {
        crate::kinfo!("(VirtIO-Net) Dispositivo VirtIO-Net encontrado");

        // TODO: Inicializar dispositivo real
        crate::kwarn!("(VirtIO-Net) Inicialização não implementada");
    }

    crate::kinfo!("(VirtIO-Net) Driver inicializado (stub)");
}

/// Desliga driver.
pub fn shutdown() {
    crate::kinfo!("(VirtIO-Net) Shutdown");
}

/// Retorna número de dispositivos.
pub fn device_count() -> usize {
    DEVICES.lock().len()
}
