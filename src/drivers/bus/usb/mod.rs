//! # Universal Serial Bus (USB) Subsystem
//!
//! O UsbBus é o barramento lógico que abstrai todos os dispositivos USB conectados.
//! Ele orquestra os Host Controllers (HCDs), realiza a enumeração universal 
//! e o pareamento com drivers funcionais (Mass Storage, HID, etc).

pub mod types;
pub mod host;
pub mod xhci;

use alloc::vec::Vec;
use alloc::sync::Arc;
use crate::sync::Spinlock;
use super::super::base::bus::{Bus, BusType, BusAddress};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::DeviceType;
use self::host::UsbHostController;

/// Gerenciador Universal de USB
pub struct UsbBus {
    /// Lista de controladores físicos registrados (xHCI, EHCI)
    hosts: Spinlock<Vec<Arc<dyn UsbHostController>>>,
}

impl UsbBus {
    pub const fn new() -> Self {
        Self {
            hosts: Spinlock::new(Vec::new()),
        }
    }

    /// Registra um novo controlador host no barramento USB
    pub fn register_host(&self, host: Arc<dyn UsbHostController>) {
        crate::kinfo!("(USB) Novo Host Controller registrado:", host.name());
        self.hosts.lock().push(host);
    }
}

impl Bus for UsbBus {
    fn name(&self) -> &'static str {
        "Universal Serial Bus (USB) Manager"
    }

    fn bus_type(&self) -> BusType {
        BusType::Usb
    }

    /// O UsbBus percorre todos os controladores registrados e perfura as portas
    /// em busca de novos dispositivos.
    fn scan(&self) -> Vec<Device> {
        let mut found_devices = Vec::new();
        let hosts = self.hosts.lock();

        crate::ktrace!("(USB) Iniciando scan em todos os hosts registrados...");

        for host in hosts.iter() {
            let events = host.poll_ports();
            
            for event in events.iter().filter(|e| e.connected) {
                crate::kinfo!("(USB) Dispositivo detectado na porta:", event.port_id as u64);
                
                // Realizar o "Universal Handshake"
                match host.setup_device(event.port_id) {
                    Ok(desc) => {
                        let dev = Device::new(
                            DeviceId(desc.id_vendor as u64 << 16 | desc.id_product as u64),
                            "USB-Device",
                            BusType::Usb,
                            BusAddress::Usb { hub_addr: 0, port: event.port_id },
                            self.map_class_to_type(desc.device_class),
                        );

                        // TODO: Armazenar descriptor nos dados privados do dispositivo
                        // dev.set_data(desc); 
                        
                        found_devices.push(dev);
                    }
                    Err(e) => {
                        crate::kerror!("(USB) Falha ao analisar dispositivo na porta:", event.port_id as u64);
                        crate::kdebug!("  -> Motivo:", 0xEE); // TODO: log enum
                    }
                }
            }
        }

        found_devices
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // TODO: Localizar o host responsável e resetar a porta correspondente
        false
    }
}

impl UsbBus {
    /// Mapeia as classes USB para tipos de dispositivos RedstoneOS
    fn map_class_to_type(&self, class: u8) -> DeviceType {
        match class {
            0x00 => DeviceType::Unknown,    // Definido via Interface Descriptor
            0x03 => DeviceType::Input,      // HID
            0x08 => DeviceType::Storage,    // Mass Storage
            0x09 => DeviceType::Controller, // Hub
            0x01 => DeviceType::Audio,      // Audio
            0x02 => DeviceType::Network,    // Communication (CDC)
            0x0E => DeviceType::Display,    // Video
            _ => DeviceType::Unknown,
        }
    }
}

// Singleton global do barramento USB
static USB_BUS_INSTANCE: UsbBus = UsbBus::new();

/// Inicializa o subsistema USB central
pub fn init() {
    crate::kdebug!("(USB) Inicializando barramento lógico universal...");
    
    // 1. Inicializar os drivers de hardware conhecidos (como xHCI)
    // Eles se registrarão no USB_BUS_INSTANCE automaticamente.
    xhci::init();

    // 2. Registra-se como um barramento disponível no RDM
    // crate::drivers::base::bus::register(Arc::new(USB_BUS_INSTANCE));
}

/// Permite que um driver de hardware (HCD) se registre no barramento central
pub fn register_hcd(host: Arc<dyn UsbHostController>) {
    USB_BUS_INSTANCE.register_host(host);
}
