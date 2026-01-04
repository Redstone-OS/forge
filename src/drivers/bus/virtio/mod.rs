//! # VirtIO Virtual Bus
//!
//! O VirtIO funciona como um barramento lógico em cima de transportes físicos
//! (PCI ou MMIO). Este módulo orquestra a descoberta de funções VirtIO.

pub mod pci;
pub mod transport;
pub mod types;
pub mod virtqueue;

use self::transport::VirtioTransport;
use super::super::base::bus::{Bus, BusAddress, BusType};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::{DeviceType, Driver, DriverError};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Gerenciador do Barramento Virtual VirtIO
pub struct VirtioBus {
    /// Transportes ativos (instâncias físicas ligadas a funções virtuais)
    transports: Spinlock<Vec<Arc<dyn VirtioTransport>>>,
}

impl VirtioBus {
    pub const fn new() -> Self {
        Self {
            transports: Spinlock::new(Vec::new()),
        }
    }

    /// Registra um novo transporte detectado (geralmente via PCI Driver)
    pub fn register_transport(&self, transport: Arc<dyn VirtioTransport>) {
        self.transports.lock().push(transport);
    }
}

impl Bus for VirtioBus {
    fn name(&self) -> &'static str {
        "VirtIO Virtual Messaging Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Virtio
    }

    /// O VirtIO Bus "escaneia" os transportes registrados e cria dispositivos
    /// RDM baseados na função (Block, Network, etc).
    fn scan(&self) -> Vec<Device> {
        let mut devices = Vec::new();
        let transports = self.transports.lock();

        for t in transports.iter() {
            let virtio_id = t.device_id();

            let dev = Device::new(
                DeviceId(0x1AF4 << 16 | self.map_virtio_id_to_num(virtio_id) as u64),
                "VirtIO-Device",
                BusType::Virtio,
                BusAddress::None, // Endereço é o transporte opaco
                self.map_id_to_dev_type(virtio_id),
            );

            // TODO: dev.set_data(t.clone());
            devices.push(dev);
        }

        devices
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // Reset via transport interface
        false
    }
}

impl VirtioBus {
    fn map_virtio_id_to_num(&self, id: types::VirtioDeviceId) -> u32 {
        match id {
            types::VirtioDeviceId::BlockDevice => 2,
            types::VirtioDeviceId::NetworkCard => 1,
            types::VirtioDeviceId::GpuDevice => 16,
            types::VirtioDeviceId::Unknown(n) => n,
            _ => 0,
        }
    }

    fn map_id_to_dev_type(&self, id: types::VirtioDeviceId) -> DeviceType {
        match id {
            types::VirtioDeviceId::BlockDevice => DeviceType::Storage,
            types::VirtioDeviceId::NetworkCard => DeviceType::Network,
            types::VirtioDeviceId::GpuDevice => DeviceType::Display,
            types::VirtioDeviceId::Console => DeviceType::Serial,
            _ => DeviceType::Unknown,
        }
    }
}

static VIRTIO_BUS_INSTANCE: VirtioBus = VirtioBus::new();

/// Inicializa o subsistema VirtIO
pub fn init() {
    crate::kdebug!("(VirtIO) Inicializando barramento virtual...");

    // Registrar o driver de transporte (Virtio-PCI) no DriverManager principal
    // para que ele possa encontrar o hardware e registrar transportes aqui.
    // crate::drivers::base::register_driver(Arc::new(pci::VirtioPciDriver));
}
