//! # VirtIO Bus Driver
//!
//! Este módulo implementa o suporte ao **VirtIO** - o framework de
//! paravirtualização usado por QEMU, KVM, e outros hypervisors.
//!
//! ## Por que VirtIO?
//! - **Performance**: Evita emulação de hardware real
//! - **Simplicidade**: Interface padronizada e simples
//! - **Portabilidade**: Mesmo driver para qualquer hypervisor
//!
//! ## Dispositivos VirtIO Suportados:
//! - **virtio-blk**: Block device (disco)
//! - **virtio-net**: Network device
//! - **virtio-gpu**: GPU 2D/3D
//! - **virtio-input**: Teclado/Mouse
//! - **virtio-console**: Console serial
//!
//! ## Transports:
//! - **PCI**: Dispositivos VirtIO aparecem como PCI devices
//! - **MMIO**: Endereços fixos de memória (usado em ARM)
//!
//! ## Arquitetura:
//! ```text
//! +----------------+
//! | virtio-blk     |  <- Drivers de dispositivo
//! | virtio-net     |
//! +-------+--------+
//!         |
//! +-------v--------+
//! | VirtIO Core    |  <- Este módulo
//! | (nego, config) |
//! +-------+--------+
//!         |
//! +-------v--------+
//! | Transport      |  <- PCI ou MMIO
//! | (queues, IRQs) |
//! +----------------+
//! ```

pub mod transport; // Camada de transporte (PCI/MMIO)
pub mod types; // Tipos e constantes VirtIO
pub mod virtqueue; // Implementação de VirtQueues

// Re-exports
pub use transport::VirtioTransport;
pub use types::*;
pub use virtqueue::VirtQueue;

use crate::drivers::base::bus::{Bus, BusAddress, BusType};
use crate::drivers::base::device::Device;
use crate::drivers::base::driver::DeviceType;
use crate::drivers::bus::pci;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Vendor ID da VirtIO (Red Hat).
pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// Device ID base para dispositivos VirtIO (transitional).
pub const VIRTIO_DEVICE_ID_BASE: u16 = 0x1000;

/// Device ID para dispositivos VirtIO 1.0+.
pub const VIRTIO_DEVICE_ID_MODERN_BASE: u16 = 0x1040;

// =============================================================================
// ESTRUTURA DE DISPOSITIVO VIRTIO
// =============================================================================

/// Representa um dispositivo VirtIO descoberto.
#[derive(Debug, Clone)]
pub struct VirtioDevice {
    /// Tipo de dispositivo (Block, Net, GPU, etc).
    pub device_type: VirtioDeviceType,

    /// Transport que conecta o dispositivo.
    pub transport: VirtioTransportType,

    /// Endereço do dispositivo (PCI BDF ou MMIO base).
    pub address: VirtioAddress,

    /// Features negociadas com o dispositivo.
    pub features: u64,

    /// Número de virtqueues.
    pub num_queues: u16,

    /// Status do dispositivo.
    pub status: u8,
}

/// Tipo de dispositivo VirtIO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioDeviceType {
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4, // RNG
    Balloon = 5,
    IoMemory = 6,
    Rpmsg = 7,
    Scsi = 8,
    Transport9P = 9,
    Mac80211 = 10,
    SerialPort = 11,
    Caif = 12,
    Memory = 13,
    Gpu = 16,
    Clock = 17,
    Input = 18,
    Vsock = 19,
    Crypto = 20,
    Signal = 21,
    PmemPersistentMem = 22,
    Iommu = 23,
    Sound = 25,
    Unknown = 0xFF,
}

impl VirtioDeviceType {
    /// Converte device ID para tipo.
    pub fn from_device_id(id: u16) -> Self {
        // Transitional (0x1000-0x103F) ou Modern (0x1040+)
        let type_id = if id >= VIRTIO_DEVICE_ID_MODERN_BASE {
            id - VIRTIO_DEVICE_ID_MODERN_BASE
        } else if id >= VIRTIO_DEVICE_ID_BASE {
            id - VIRTIO_DEVICE_ID_BASE
        } else {
            return Self::Unknown;
        };

        match type_id {
            1 => Self::Network,
            2 => Self::Block,
            3 => Self::Console,
            4 => Self::Entropy,
            5 => Self::Balloon,
            8 => Self::Scsi,
            16 => Self::Gpu,
            18 => Self::Input,
            25 => Self::Sound,
            _ => Self::Unknown,
        }
    }

    /// Retorna nome legível.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Network => "VirtIO Network",
            Self::Block => "VirtIO Block",
            Self::Console => "VirtIO Console",
            Self::Entropy => "VirtIO RNG",
            Self::Gpu => "VirtIO GPU",
            Self::Input => "VirtIO Input",
            Self::Sound => "VirtIO Sound",
            Self::Scsi => "VirtIO SCSI",
            _ => "VirtIO Unknown",
        }
    }

    /// Converte para DeviceType da base.
    pub fn to_device_type(&self) -> DeviceType {
        match self {
            Self::Block | Self::Scsi => DeviceType::Storage,
            Self::Network => DeviceType::Network,
            Self::Gpu => DeviceType::Display,
            Self::Input => DeviceType::Input,
            Self::Sound => DeviceType::Audio,
            Self::Entropy => DeviceType::Security,
            Self::Console => DeviceType::Serial,
            _ => DeviceType::Generic,
        }
    }
}

/// Tipo de transport VirtIO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioTransportType {
    Pci,
    Mmio,
}

/// Endereço de dispositivo VirtIO.
#[derive(Debug, Clone, Copy)]
pub enum VirtioAddress {
    Pci(pci::PciAddress),
    Mmio(u64),
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos VirtIO descobertos.
static VIRTIO_DEVICES: Spinlock<Vec<VirtioDevice>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// IMPLEMENTAÇÃO DO BUS TRAIT
// =============================================================================

/// Implementação do barramento VirtIO.
pub struct VirtioBus;

impl Bus for VirtioBus {
    fn name(&self) -> &'static str {
        "VirtIO Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Virtio
    }

    fn scan(&self) -> Vec<Device> {
        scan()
    }

    fn reset_device(&self, dev: &mut Device) -> bool {
        crate::kwarn!("(VirtIO) reset_device() - usando reset de status");

        // Reset VirtIO: escreve 0 no status
        // TODO: Implementar via transport
        true
    }

    fn shutdown(&self) {
        crate::kinfo!("(VirtIO) Desligando dispositivos VirtIO...");
    }
}

/// Retorna referência ao barramento VirtIO.
pub fn get_bus() -> Arc<dyn Bus> {
    Arc::new(VirtioBus)
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema VirtIO.
pub fn init() {
    crate::kinfo!("(VirtIO) Inicializando suporte VirtIO...");

    *INITIALIZED.lock() = true;

    // Registra na base
    crate::drivers::base::bus::register(Arc::new(VirtioBus));

    crate::kinfo!("(VirtIO) Subsistema inicializado");
}

/// Escaneia por dispositivos VirtIO.
///
/// Procura:
/// 1. Dispositivos PCI com Vendor ID 0x1AF4
/// 2. Regiões MMIO conhecidas (devicetree/ACPI)
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(VirtIO) Escaneando dispositivos VirtIO...");

    let mut devices = Vec::new();
    let mut virtio_devices = VIRTIO_DEVICES.lock();
    virtio_devices.clear();

    // Escaneia dispositivos PCI VirtIO
    scan_pci_virtio(&mut *virtio_devices);

    // TODO: Escaneia MMIO VirtIO (para ARM ou devicetree)

    crate::kinfo!("(VirtIO) Encontrados", virtio_devices.len(), "dispositivos");

    // Converte para Device da base
    for vdev in virtio_devices.iter() {
        let dev = virtio_device_to_base_device(vdev);
        devices.push(dev);
    }

    devices
}

/// Escaneia dispositivos VirtIO via PCI.
fn scan_pci_virtio(devices: &mut Vec<VirtioDevice>) {
    // Busca todos os dispositivos PCI com vendor ID VirtIO
    let pci_devices = pci::PCI_DEVICES.lock();

    for pci_dev in pci_devices.iter() {
        if pci_dev.vendor_id != VIRTIO_VENDOR_ID {
            continue;
        }

        let vtype = VirtioDeviceType::from_device_id(pci_dev.device_id);

        crate::kinfo!("(VirtIO) Encontrado via PCI:", vtype.as_str());

        let vdev = VirtioDevice {
            device_type: vtype,
            transport: VirtioTransportType::Pci,
            address: VirtioAddress::Pci(pci_dev.address),
            features: 0, // Será negociado depois
            num_queues: 0,
            status: 0,
        };

        devices.push(vdev);
    }
}

/// Converte VirtioDevice para Device da base.
fn virtio_device_to_base_device(vdev: &VirtioDevice) -> Device {
    let bus_addr = match vdev.address {
        VirtioAddress::Pci(addr) => addr.to_bus_address(),
        VirtioAddress::Mmio(base) => BusAddress::Mmio { base, size: 0x1000 },
    };

    let mut dev = Device::new(
        vdev.device_type.as_str(),
        BusType::Virtio,
        bus_addr,
        vdev.device_type.to_device_type(),
    );

    dev.set_ids(
        VIRTIO_VENDOR_ID,
        vdev.device_type as u16,
        0, // class
        0, // subclass
        0, // revision
    );

    dev
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Retorna número de dispositivos VirtIO.
pub fn device_count() -> usize {
    VIRTIO_DEVICES.lock().len()
}

/// Busca dispositivo VirtIO por tipo.
pub fn find_by_type(vtype: VirtioDeviceType) -> Option<VirtioDevice> {
    VIRTIO_DEVICES
        .lock()
        .iter()
        .find(|d| d.device_type == vtype)
        .cloned()
}

/// Desliga subsistema VirtIO.
pub fn shutdown() {
    crate::kinfo!("(VirtIO) Shutdown");
}
