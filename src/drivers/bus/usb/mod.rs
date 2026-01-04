//! # USB Stack - Universal Serial Bus
//!
//! Este módulo implementa o stack USB do RedstoneOS. O USB é um barramento
//! hierárquico versátil usado por milhares de tipos de dispositivos.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────┐
//! │          USB Device Drivers         │  (HID, Storage, Audio, etc)
//! ├─────────────────────────────────────┤
//! │               USB Core              │  (este módulo)
//! ├────────────┬────────────┬───────────┤
//! │      xHCI  │   EHCI     │   OHCI    │  (Host Controllers)
//! ├────────────┴────────────┴───────────┤
//! │                 PCI                 │  (Transport)
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Host Controllers:
//! - **xHCI**: USB 3.x (SuperSpeed, SuperSpeed+) - PRINCIPAL
//! - **EHCI**: USB 2.0 (High Speed) - Legado
//! - **OHCI/UHCI**: USB 1.x (Full/Low Speed) - Muito antigo
//!
//! ## Tipos de Transfer:
//! - **Control**: Configuração do dispositivo
//! - **Bulk**: Dados grandes, não urgentes (storage)
//! - **Interrupt**: Dados pequenos, periódicos (HID)
//! - **Isochronous**: Tempo real (áudio, vídeo)
//!
//! ## STUB:
//! Stack parcialmente implementado. xHCI tem estruturas básicas,
//! drivers de dispositivo são stubs.

pub mod device; // Estrutura de dispositivo USB
pub mod host;
pub mod types; // Tipos e constantes USB // Interface de host controller

// Host Controllers
pub mod ehci;
pub mod xhci; // xHCI (USB 3.0+) // EHCI (USB 2.0)

// Device Drivers
pub mod mass_storage; // USB Mass Storage

// Re-exports
pub use device::UsbDevice;
pub use types::*;

use crate::drivers::base::bus::{Bus, BusType};
use crate::drivers::base::device::Device;
use crate::drivers::bus::pci;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de host controllers detectados.
static HOST_CONTROLLERS: Spinlock<Vec<HostControllerInfo>> = Spinlock::new(Vec::new());

/// Lista de dispositivos USB conectados.
static USB_DEVICES: Spinlock<Vec<UsbDevice>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

/// Informações de um host controller.
#[derive(Debug, Clone)]
pub struct HostControllerInfo {
    pub controller_type: HostControllerType,
    pub pci_address: Option<pci::PciAddress>,
    pub mmio_base: u64,
    pub irq: u8,
    pub max_ports: u8,
}

/// Tipo de host controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostControllerType {
    Xhci, // USB 3.0+
    Ehci, // USB 2.0
    Ohci, // USB 1.1
    Uhci, // USB 1.0
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o stack USB.
pub fn init() {
    crate::kinfo!("(USB) Inicializando USB Stack...");

    // Detecta host controllers via PCI
    detect_host_controllers();

    // Inicializa xHCI primeiro (mais moderno)
    xhci::init();

    // EHCI como fallback
    ehci::init();

    *INITIALIZED.lock() = true;

    crate::kinfo!("(USB) Stack inicializado");
}

/// Detecta host controllers USB via PCI.
fn detect_host_controllers() {
    let pci_devices = pci::PCI_DEVICES.lock();
    let mut controllers = HOST_CONTROLLERS.lock();

    for pci_dev in pci_devices.iter() {
        // USB controllers: class=0x0C, subclass=0x03
        if pci_dev.class_code != 0x0C || pci_dev.subclass_code != 0x03 {
            continue;
        }

        let hc_type = match pci_dev.prog_if {
            pci::config::PCI_PROG_IF_XHCI => HostControllerType::Xhci,
            pci::config::PCI_PROG_IF_EHCI => HostControllerType::Ehci,
            pci::config::PCI_PROG_IF_OHCI => HostControllerType::Ohci,
            pci::config::PCI_PROG_IF_UHCI => HostControllerType::Uhci,
            _ => continue,
        };

        crate::kinfo!(
            "(USB) Host Controller encontrado:",
            match hc_type {
                HostControllerType::Xhci => "xHCI",
                HostControllerType::Ehci => "EHCI",
                HostControllerType::Ohci => "OHCI",
                HostControllerType::Uhci => "UHCI",
            }
        );

        // Obtém BAR0 (MMIO base)
        let mmio_base = pci_dev.bar_address(0);

        controllers.push(HostControllerInfo {
            controller_type: hc_type,
            pci_address: Some(pci_dev.address),
            mmio_base,
            irq: pci_dev.interrupt_line,
            max_ports: 0, // Será detectado depois
        });
    }

    crate::kinfo!("(USB) Detectados", controllers.len(), "host controllers");
}

/// Escaneia por dispositivos USB conectados.
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(USB) Escaneando dispositivos USB...");

    // TODO: Implementar enumeration real
    crate::kwarn!("(USB) scan() não totalmente implementado");

    Vec::new()
}

/// Retorna número de dispositivos USB.
pub fn device_count() -> usize {
    USB_DEVICES.lock().len()
}

/// Desliga o stack USB.
pub fn shutdown() {
    crate::kinfo!("(USB) Shutdown do stack USB...");

    xhci::shutdown();
    ehci::shutdown();
}

// =============================================================================
// FUNÇÕES DE REGISTRO DE DISPOSITIVO
// =============================================================================

/// Registra um novo dispositivo USB descoberto.
///
/// Chamado pelo host controller quando um novo dispositivo é conectado.
pub fn register_device(device: UsbDevice) {
    crate::kinfo!(
        "(USB) Novo dispositivo:",
        device.vendor_id,
        ":",
        device.product_id
    );

    USB_DEVICES.lock().push(device);
}

/// Remove um dispositivo USB (desconectado).
pub fn unregister_device(address: u8) {
    crate::kinfo!("(USB) Dispositivo removido, addr:", address);
    USB_DEVICES.lock().retain(|d| d.address != address);
}

// =============================================================================
// FUNÇÕES DE CONSULTA
// =============================================================================

/// Retorna lista de host controllers.
pub fn get_host_controllers() -> Vec<HostControllerInfo> {
    HOST_CONTROLLERS.lock().clone()
}

/// Verifica se há algum xHCI disponível.
pub fn has_xhci() -> bool {
    HOST_CONTROLLERS
        .lock()
        .iter()
        .any(|hc| hc.controller_type == HostControllerType::Xhci)
}
