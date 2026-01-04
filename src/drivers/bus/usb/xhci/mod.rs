//! # xHCI (eXtensible Host Controller Interface)
//!
//! Driver para controladores USB 3.0+.
//! Este módulo implementa a trait UsbHostController para o USB Core.

pub mod controller;
pub mod device;
pub mod global;
pub mod port;
pub mod regs;
pub mod ring;
pub mod structs;
pub mod transfer;
pub mod types;

use self::controller::XhciController;
use super::super::base::device::{Device, DeviceState};
use super::super::base::driver::{DeviceType, Driver, DriverError};
use super::host::{UsbDeviceDescriptor, UsbError, UsbHostController, UsbPortEvent};
use super::types::UsbSpeed as UniversalSpeed;
use crate::sync::Spinlock;
use alloc::sync::Arc;

/// Wrapper para integrar o xHCI com o RDM e UsbBus
pub struct XhciDriver;

impl Driver for XhciDriver {
    fn name(&self) -> &'static str {
        "xHCI USB 3.0 Host Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Controller
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // 1. Verificar se é um controlador USB habilitado via PCI
        // TODO: Validar Class/Subclass se necessário

        let pci_info = match dev.get_data::<crate::drivers::bus::pci::device::PciDeviceInfo>() {
            Some(info) => info,
            None => return Err(DriverError::NotSupported),
        };

        crate::kinfo!("(xHCI) Probing controlador em PCI:", dev.bus_address);

        // 2. Inicializar o controlador (isso requer transformar PciDeviceInfo em PciDevice ou similar)
        // Por enquanto, usamos a lógica existente.

        // let xhci = XhciController::new(pci_info).ok_or(DriverError::HardwareFault)?;
        // let host_adapter = Arc::new(XhciHostAdapter { inner: Spinlock::new(xhci) });

        // 3. Registrar no UsbBus
        // super::register_hcd(host_adapter);

        Ok(())
    }
}

pub struct XhciHostAdapter {
    pub inner: Spinlock<XhciController>,
}

impl UsbHostController for XhciHostAdapter {
    fn name(&self) -> &'static str {
        "xHCI Host Controller"
    }

    fn poll_ports(&self) -> Vec<UsbPortEvent> {
        let xhci = self.inner.lock();
        let mut events = Vec::new();
        for i in 1..=xhci.max_ports {
            if let Some(port) = xhci.read_port_status(i) {
                if port.connected {
                    events.push(UsbPortEvent {
                        port_id: i,
                        connected: true,
                        speed: match port.speed {
                            super::xhci::types::UsbSpeed::Low => UniversalSpeed::LowSpeed,
                            super::xhci::types::UsbSpeed::Full => UniversalSpeed::FullSpeed,
                            super::xhci::types::UsbSpeed::High => UniversalSpeed::HighSpeed,
                            super::xhci::types::UsbSpeed::Super => UniversalSpeed::SuperSpeed,
                            super::xhci::types::UsbSpeed::SuperPlus => {
                                UniversalSpeed::SuperSpeedPlus
                            }
                        },
                    });
                }
            }
        }
        events
    }

    fn setup_device(&self, port_id: u8) -> Result<UsbDeviceDescriptor, UsbError> {
        // TODO: Migrar lógica de address_device e read_descriptor para cá
        Err(UsbError::HardwareError)
    }

    fn transfer(&self, _request: super::host::UsbTransferRequest) -> Result<(), UsbError> {
        Ok(())
    }
}

pub fn init() {
    crate::kdebug!("(xHCI) Registrando driver de hardware no RDM...");
    crate::drivers::base::register_driver(Arc::new(XhciDriver));
}
