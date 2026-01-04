//! # Inicialização do xHCI
//!
//! Gerenciamento global do driver xHCI.

use crate::drivers::pci;
use crate::sync::Spinlock;
use alloc::vec::Vec;

use super::controller::XhciController;
use super::regs;
use super::types::UsbPort;

/// Controlador global protegida por Spinlock
static XHCI_CONTROLLER: Spinlock<Option<XhciController>> = Spinlock::new(None);

/// Inicializa o driver xHCI
///
/// Procura por controladores xHCI no barramento PCI e inicializa o primeiro encontrado.
pub fn init() -> bool {
    crate::kinfo!("(xHCI) Procurando controlador USB 3.0...");

    // Procurar controlador xHCI no PCI
    // Class 0x0C (Serial Bus), Subclass 0x03 (USB), ProgIF 0x30 (xHCI)
    let pci_device = match pci::find_by_class(
        regs::PCI_CLASS_SERIAL_BUS,
        regs::PCI_SUBCLASS_USB,
        Some(regs::PCI_PROG_IF_XHCI),
    ) {
        Some(dev) => dev,
        None => {
            crate::kwarn!("(xHCI) Nenhum controlador encontrado");
            return false;
        }
    };

    crate::kinfo!("(xHCI) Controlador encontrado!");
    // crate::kinfo!("  Bus: {}, Device: {}, Function: {}", pci_device.bus, pci_device.device, pci_device.function);

    // Inicializar controlador
    let controller = match XhciController::new(pci_device) {
        Some(c) => c,
        None => {
            crate::kerror!("(xHCI) Falha ao inicializar controlador");
            return false;
        }
    };

    *XHCI_CONTROLLER.lock() = Some(controller);

    crate::kinfo!("(xHCI) Driver inicializado!");
    true
}

/// Escaneia portas USB e retorna dispositivos conectados
pub fn scan_ports() -> Vec<UsbPort> {
    let mut guard = XHCI_CONTROLLER.lock();
    if let Some(ref mut controller) = *guard {
        controller.scan_ports()
    } else {
        Vec::new()
    }
}

/// Verifica se o driver está inicializado
pub fn is_initialized() -> bool {
    XHCI_CONTROLLER.lock().is_some()
}

/// Executa uma função com o controlador xHCI bloqueado
pub fn with_controller<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut XhciController) -> R,
{
    let mut guard = XHCI_CONTROLLER.lock();
    guard.as_mut().map(|controller| f(controller))
}
