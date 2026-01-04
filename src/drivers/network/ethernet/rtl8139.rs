//! # Realtek RTL8139 Ethernet Driver
//!
//! Driver para o adaptador **Realtek RTL8139** - uma NIC Fast Ethernet
//! muito comum e simples.
//!
//! ## Características:
//! - 10/100 Mbps Fast Ethernet
//! - PCI
//! - Muito simples de implementar
//! - Comum em hardware antigo e algumas VMs
//!
//! ## PCI IDs:
//! - Vendor: 0x10EC (Realtek)
//! - Device: 0x8139
//!
//! ## STUB:
//! Estrutura definida. Implementação pendente.

use crate::drivers::bus::pci::{PciAddress, PciDevice};
use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Realtek Vendor ID.
pub const REALTEK_VENDOR_ID: u16 = 0x10EC;

/// RTL8139 Device ID.
pub const RTL8139_DEVICE_ID: u16 = 0x8139;

// =============================================================================
// REGISTRADORES RTL8139
// =============================================================================

/// MAC Address.
pub const RTL_MAC0: u32 = 0x00;
/// Multicast Address.
pub const RTL_MAR0: u32 = 0x08;
/// Transmit Status (4 descriptors).
pub const RTL_TSD0: u32 = 0x10;
/// Transmit Start Address.
pub const RTL_TSAD0: u32 = 0x20;
/// Receive Buffer Start Address.
pub const RTL_RBSTART: u32 = 0x30;
/// Command Register.
pub const RTL_CMD: u32 = 0x37;
/// Current Address of Packet Read.
pub const RTL_CAPR: u32 = 0x38;
/// Current Buffer Address.
pub const RTL_CBR: u32 = 0x3A;
/// Interrupt Mask Register.
pub const RTL_IMR: u32 = 0x3C;
/// Interrupt Status Register.
pub const RTL_ISR: u32 = 0x3E;
/// Transmit Configuration.
pub const RTL_TCR: u32 = 0x40;
/// Receive Configuration.
pub const RTL_RCR: u32 = 0x44;
/// Configuration Register 1.
pub const RTL_CONFIG1: u32 = 0x52;

// =============================================================================
// ESTRUTURA DO DRIVER
// =============================================================================

// TODO: Revisar no futuro
#[allow(unused)]
/// Driver RTL8139.
pub struct Rtl8139Device {
    /// Endereço PCI.
    pci_address: PciAddress,

    /// Porta I/O base.
    io_base: u16,

    /// IRQ.
    irq: u8,

    /// Habilitado?
    enabled: Spinlock<bool>,
}

impl Rtl8139Device {
    /// Probe de dispositivo PCI.
    ///
    /// ## STUB:
    /// Não implementado.
    pub fn probe(pci_dev: &PciDevice) -> Option<Self> {
        if pci_dev.vendor_id != REALTEK_VENDOR_ID {
            return None;
        }

        if pci_dev.device_id == RTL8139_DEVICE_ID {
            crate::kinfo!("(RTL8139) Dispositivo Realtek RTL8139 encontrado!");
            crate::kwarn!("(RTL8139) Inicialização não implementada");
            return None;
        }

        None
    }
}

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Busca e inicializa dispositivos RTL8139.
pub fn detect_and_init() {
    crate::kinfo!("(RTL8139) Buscando dispositivos...");
    crate::kwarn!("(RTL8139) Detecção não implementada");
}
