//! # PCI / PCI Express Bus
//!
//! Implementação do barramento PCI (Peripheral Component Interconnect).
//! Responsável pela enumeração dinâmica de dispositivos no barramento.

pub mod access;
pub mod device;

use alloc::vec::Vec;
use crate::sync::Spinlock;
use super::super::base::bus::{Bus, BusType, BusAddress};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::DeviceType;
use self::device::PciDeviceInfo;

pub struct PciBus {
    /// Lista de dispositivos brutos encontrados (Cache de enumeração)
    discovered_raw: Spinlock<Vec<PciDeviceInfo>>,
}

impl PciBus {
    pub const fn new() -> Self {
        Self {
            discovered_raw: Spinlock::new(Vec::new()),
        }
    }
}

impl Bus for PciBus {
    fn name(&self) -> &'static str {
        "PCI/PCIe Bus Manager"
    }

    fn bus_type(&self) -> BusType {
        BusType::Pci
    }

    /// Escaneia o barramento PCI procurando por dispositivos
    fn scan(&self) -> Vec<Device> {
        let mut devices = Vec::new();
        let mut raw_cache = self.discovered_raw.lock();
        raw_cache.clear();

        crate::kinfo!("(PCI) Iniciando enumeração do barramento...");

        for bus in 0..=255u8 {
            for slot in 0..32u8 {
                // Verificar função 0
                if let Some(info) = PciDeviceInfo::read(bus, slot, 0) {
                    let is_multi = (info.header_type & 0x80) != 0;
                    
                    self.register_found_device(&mut devices, &mut raw_cache, info);

                    // Se for multi-função, verificar 1-7
                    if is_multi {
                        for func in 1..8u8 {
                            if let Some(info) = PciDeviceInfo::read(bus, slot, func) {
                                self.register_found_device(&mut devices, &mut raw_cache, info);
                            }
                        }
                    }
                }
            }
            
            // Otimização: se não houver nada no bus 0, geralmente não há mais barramentos
            if bus == 0 && raw_cache.is_empty() { break; }
        }

        crate::kinfo!("(PCI) Enumeração concluída. Dispositivos:", raw_cache.len() as u64);
        devices
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // TODO: Implementar reset via Secondary Bus Reset ou FLR (Function Level Reset)
        crate::kwarn!("(PCI) Reset de dispositivo PCI solicitado, mas não implementado.");
        false
    }
}

impl PciBus {
    /// Transforma uma estrutura técnica PCI em um objeto Device do RDM
    fn register_found_device(&self, devices: &mut Vec<Device>, cache: &mut Vec<PciDeviceInfo>, info: PciDeviceInfo) {
        let mut dev = Device::new(
            DeviceId(info.vendor_id as u64 << 16 | info.device_id as u64),
            "PCI-Device",
            BusType::Pci,
            BusAddress::Pci { 
                bus: info.bus, 
                dev: info.device, 
                func: info.function 
            },
            self.map_class_to_dev_type(info.class_code),
        );

        // Anexar informações técnicas como dado privado
        dev.set_data(info);
        
        devices.push(dev);
        cache.push(info);
    }

    /// Mapeia Class Codes do PCI para DeviceType do RedstoneOS
    fn map_class_to_dev_type(&self, class: u8) -> DeviceType {
        match class {
            0x01 => DeviceType::Storage,    // Mass Storage Controller
            0x02 => DeviceType::Network,    // Network Controller
            0x03 => DeviceType::Display,    // Display Controller
            0x04 => DeviceType::Audio,      // Multimedia Controller
            0x06 => DeviceType::Bus,        // Bridge Device
            0x0C => DeviceType::Controller, // Serial Bus Controller (USB, Firewire)
            0x08 => DeviceType::Controller, // Generic System Peripheral
            _ => DeviceType::Unknown,
        }
    }
}

static PCI_BUS_INSTANCE: PciBus = PciBus::new();

/// Inicializa o barramento PCI
pub fn init() {
    crate::kdebug!("(PCI) Registrando barramento no RDM...");
    // Em uma versão futura, registra-se no RDM global
    // crate::drivers::base::bus::register(Arc::new(PCI_BUS_INSTANCE));
}
