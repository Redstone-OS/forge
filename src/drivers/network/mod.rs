//! # Network Drivers Layer
//!
//! Este módulo contém os drivers de rede do RedstoneOS. Segue a arquitetura
//! padrão do RDS para integração com o stack de rede.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           TCP/IP Stack                  │  (Futuro)
//! ├─────────────────────────────────────────┤
//! │           Network Core                  │  (netdev, buffers)
//! ├─────────────────────────────────────────┤
//! │            Network Drivers              │  (este módulo)
//! ├──────────┬──────────┬──────────┬────────┤
//! │  VirtIO  │   e1000  │  RTL8139 │  WiFi  │
//! ├──────────┴──────────┴──────────┴────────┤
//! │           PCI / Platform                │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Drivers Planejados:
//!
//! ### Críticos (Fase 1):
//! - **VirtIO-Net**: Paravirtualização para QEMU/KVM
//! - **Loopback**: Interface local (127.0.0.1)
//!
//! ### Secundários (Fases Posteriores):
//! - **e1000**: Intel Gigabit Ethernet (muito comum em VMs)
//! - **RTL8139**: Realtek Fast Ethernet (legado)
//!
//! ### Futuro:
//! - **WiFi**: 802.11 (requer subsistema wireless)
//!
//! ## STUB:
//! Network stack ainda não implementado. Este módulo define
//! a estrutura e interfaces que serão usadas.

pub mod ethernet; // Drivers Ethernet (e1000, rtl8139)
pub mod loopback; // Loopback device
pub mod mii;
pub mod traits; // Traits de dispositivo de rede
pub mod virtio; // VirtIO-Net
pub mod wifi; // WiFi (futuro) // Media Independent Interface

// Re-exports
pub use traits::*;

// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::base::bus::BusType;
use crate::drivers::base::device::Device;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos de rede registrados.
static NETWORK_DEVICES: Spinlock<Vec<Arc<dyn NetworkDevice>>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de rede.
pub fn init() {
    crate::kinfo!("(Network) Inicializando subsistema de rede...");

    // Inicializa loopback primeiro (sempre disponível)
    loopback::init();

    // Inicializa VirtIO-Net (para VMs)
    virtio::init();

    // Inicializa drivers Ethernet
    ethernet::init();

    *INITIALIZED.lock() = true;

    crate::kinfo!("(Network) Subsistema inicializado");
}

/// Escaneia por dispositivos de rede.
///
/// ## STUB:
/// Retorna lista vazia por enquanto.
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(Network) Escaneando dispositivos de rede...");

    crate::kwarn!("(Network) scan() não implementado");

    Vec::new()
}

/// Desliga o subsistema de rede.
pub fn shutdown() {
    crate::kinfo!("(Network) Shutdown do subsistema de rede...");

    ethernet::shutdown();
    virtio::shutdown();
    loopback::shutdown();
}

// =============================================================================
// REGISTRO DE DISPOSITIVOS
// =============================================================================

/// Registra um novo dispositivo de rede.
pub fn register_device(dev: Arc<dyn NetworkDevice>) {
    let name = dev.name();
    crate::kinfo!("(Network) Registrando dispositivo:", name);

    NETWORK_DEVICES.lock().push(dev);
}

/// Remove um dispositivo de rede.
pub fn unregister_device(name: &str) {
    crate::kinfo!("(Network) Removendo dispositivo:", name);
    NETWORK_DEVICES.lock().retain(|d| d.name() != name);
}

// =============================================================================
// FUNÇÕES DE CONSULTA
// =============================================================================

/// Retorna número de dispositivos de rede.
pub fn device_count() -> usize {
    NETWORK_DEVICES.lock().len()
}

/// Busca dispositivo por nome.
pub fn find_device(name: &str) -> Option<Arc<dyn NetworkDevice>> {
    NETWORK_DEVICES
        .lock()
        .iter()
        .find(|d| d.name() == name)
        .cloned()
}

/// Retorna lista de todos os dispositivos.
pub fn get_all_devices() -> Vec<Arc<dyn NetworkDevice>> {
    NETWORK_DEVICES.lock().clone()
}

/// Retorna o dispositivo padrão (primeiro ativo).
pub fn get_default_device() -> Option<Arc<dyn NetworkDevice>> {
    NETWORK_DEVICES
        .lock()
        .iter()
        .find(|d| d.is_link_up())
        .cloned()
}

// =============================================================================
// ESTATÍSTICAS GLOBAIS
// =============================================================================

/// Estatísticas globais de rede.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total de pacotes recebidos.
    pub rx_packets: u64,
    /// Total de pacotes transmitidos.
    pub tx_packets: u64,
    /// Total de bytes recebidos.
    pub rx_bytes: u64,
    /// Total de bytes transmitidos.
    pub tx_bytes: u64,
    /// Erros de recepção.
    pub rx_errors: u64,
    /// Erros de transmissão.
    pub tx_errors: u64,
    /// Pacotes descartados (RX).
    pub rx_dropped: u64,
    /// Pacotes descartados (TX).
    pub tx_dropped: u64,
}

/// Retorna estatísticas agregadas de todos os dispositivos.
pub fn get_global_stats() -> NetworkStats {
    let devices = NETWORK_DEVICES.lock();
    let mut global = NetworkStats::default();

    for dev in devices.iter() {
        let stats = dev.get_stats();
        global.rx_packets += stats.rx_packets;
        global.tx_packets += stats.tx_packets;
        global.rx_bytes += stats.rx_bytes;
        global.tx_bytes += stats.tx_bytes;
        global.rx_errors += stats.rx_errors;
        global.tx_errors += stats.tx_errors;
        global.rx_dropped += stats.rx_dropped;
        global.tx_dropped += stats.tx_dropped;
    }

    global
}
