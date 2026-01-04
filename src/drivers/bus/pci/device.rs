//! # Estrutura de Dispositivo PCI
//!
//! Este arquivo define a estrutura `PciDevice` que representa um
//! dispositivo PCI descoberto durante o scan do barramento.

use super::access::{read_config, write_config};
use super::config;
use super::PciAddress;

// =============================================================================
// ESTRUTURA PRINCIPAL
// =============================================================================

/// Representa um dispositivo PCI descoberto.
#[derive(Debug, Clone)]
pub struct PciDevice {
    /// Endereço PCI (segment:bus:device.function).
    pub address: PciAddress,

    /// Vendor ID do fabricante.
    pub vendor_id: u16,

    /// Device ID do produto.
    pub device_id: u16,

    /// Código de classe (ex: 0x01 = Storage).
    pub class_code: u8,

    /// Código de subclasse (ex: 0x06 = SATA).
    pub subclass_code: u8,

    /// Programming Interface.
    pub prog_if: u8,

    /// Revisão do hardware.
    pub revision: u8,

    /// Tipo de header (0, 1, ou 2).
    pub header_type: u8,

    /// Linha de IRQ atribuída.
    pub interrupt_line: u8,

    /// Pino de interrupção (INTA-INTD).
    pub interrupt_pin: u8,

    /// Base Address Registers (6 BARs).
    pub bars: [u32; 6],
}

impl PciDevice {
    /// Verifica se um BAR é do tipo I/O (vs Memory).
    pub fn is_bar_io(&self, bar_index: usize) -> bool {
        if bar_index >= 6 {
            return false;
        }
        (self.bars[bar_index] & 0x01) != 0
    }

    /// Retorna o tipo de BAR de memória.
    /// 0 = 32-bit, 1 = abaixo de 1MB, 2 = 64-bit
    pub fn bar_memory_type(&self, bar_index: usize) -> u8 {
        if bar_index >= 6 || self.is_bar_io(bar_index) {
            return 0;
        }
        ((self.bars[bar_index] >> 1) & 0x03) as u8
    }

    /// Retorna se o BAR é 64 bits.
    pub fn is_bar_64bit(&self, bar_index: usize) -> bool {
        self.bar_memory_type(bar_index) == 2
    }

    /// Retorna o endereço base de um BAR de memória.
    pub fn bar_address(&self, bar_index: usize) -> u64 {
        if bar_index >= 6 {
            return 0;
        }

        if self.is_bar_io(bar_index) {
            // I/O BAR: bits 1:0 são tipo, resto é endereço
            return (self.bars[bar_index] & !0x03) as u64;
        }

        // Memory BAR
        let low = self.bars[bar_index] & !0x0F;

        if self.is_bar_64bit(bar_index) && bar_index < 5 {
            // 64-bit BAR usa dois registradores consecutivos
            let high = self.bars[bar_index + 1];
            return ((high as u64) << 32) | (low as u64);
        }

        low as u64
    }

    /// Calcula o tamanho de um BAR.
    ///
    /// Algoritmo:
    /// 1. Salva valor atual
    /// 2. Escreve 0xFFFFFFFF
    /// 3. Lê de volta (bits fixos indicam tamanho)
    /// 4. Restaura valor original
    pub fn bar_size(&self, bar_index: usize) -> u64 {
        if bar_index >= 6 {
            return 0;
        }

        let bar_offset = config::PCI_BAR0 + (bar_index as u8 * 4);

        // Salva valor original
        let original = read_config(self.address, bar_offset);

        // Escreve todos 1s
        write_config(self.address, bar_offset, 0xFFFF_FFFF);

        // Lê de volta
        let size_mask = read_config(self.address, bar_offset);

        // Restaura
        write_config(self.address, bar_offset, original);

        if size_mask == 0 || size_mask == 0xFFFF_FFFF {
            return 0;
        }

        // Calcula tamanho
        if self.is_bar_io(bar_index) {
            let mask = size_mask & !0x03;
            ((!mask) + 1) as u64
        } else {
            let mask = size_mask & !0x0F;
            if mask == 0 {
                return 0;
            }
            ((!mask) + 1) as u64
        }
    }

    /// Habilita o dispositivo (I/O + Memory + Bus Master).
    pub fn enable(&self) {
        let cmd = read_config(self.address, config::PCI_COMMAND);
        let new_cmd = cmd | 0x07; // I/O + Memory + Bus Master
        write_config(self.address, config::PCI_COMMAND, new_cmd);
    }

    /// Desabilita o dispositivo.
    pub fn disable(&self) {
        let cmd = read_config(self.address, config::PCI_COMMAND);
        let new_cmd = cmd & !0x07;
        write_config(self.address, config::PCI_COMMAND, new_cmd);
    }

    /// Verifica se o dispositivo tem capabilities.
    pub fn has_capabilities(&self) -> bool {
        let status = read_config(self.address, config::PCI_STATUS) as u16;
        (status & 0x10) != 0 // Bit 4 = Capabilities List
    }

    /// Retorna o offset da primeira capability.
    pub fn capabilities_offset(&self) -> Option<u8> {
        if !self.has_capabilities() {
            return None;
        }
        let ptr = read_config(self.address, config::PCI_CAPABILITIES_PTR) as u8;
        if ptr != 0 {
            Some(ptr)
        } else {
            None
        }
    }

    /// Busca uma capability específica.
    ///
    /// Retorna o offset da capability encontrada, ou None.
    pub fn find_capability(&self, cap_id: u8) -> Option<u8> {
        let mut offset = self.capabilities_offset()?;

        // Limite de iterações para evitar loop infinito
        for _ in 0..48 {
            if offset == 0 || offset == 0xFF {
                return None;
            }

            let cap_header = read_config(self.address, offset);
            let current_id = (cap_header & 0xFF) as u8;

            if current_id == cap_id {
                return Some(offset);
            }

            // Próxima capability
            offset = ((cap_header >> 8) & 0xFF) as u8;
        }

        None
    }

    /// Verifica se é ponte PCI-to-PCI.
    pub fn is_bridge(&self) -> bool {
        self.class_code == 0x06 && self.subclass_code == 0x04
    }

    /// Verifica se é controlador USB.
    pub fn is_usb_controller(&self) -> bool {
        self.class_code == 0x0C && self.subclass_code == 0x03
    }

    /// Verifica se suporta MSI.
    pub fn supports_msi(&self) -> bool {
        self.find_capability(0x05).is_some() // MSI cap ID = 0x05
    }

    /// Verifica se suporta MSI-X.
    pub fn supports_msi_x(&self) -> bool {
        self.find_capability(0x11).is_some() // MSI-X cap ID = 0x11
    }
}

// =============================================================================
// CAPABILITY IDS
// =============================================================================

/// Power Management Capability.
pub const PCI_CAP_PM: u8 = 0x01;

/// AGP Capability.
pub const PCI_CAP_AGP: u8 = 0x02;

/// Vital Product Data.
pub const PCI_CAP_VPD: u8 = 0x03;

/// Slot Identification.
pub const PCI_CAP_SLOTID: u8 = 0x04;

/// MSI Capability.
pub const PCI_CAP_MSI: u8 = 0x05;

/// CompactPCI Hot Swap.
pub const PCI_CAP_CHSWP: u8 = 0x06;

/// PCI-X Capability.
pub const PCI_CAP_PCIX: u8 = 0x07;

/// HyperTransport.
pub const PCI_CAP_HT: u8 = 0x08;

/// Vendor Specific.
pub const PCI_CAP_VENDOR: u8 = 0x09;

/// Debug Port.
pub const PCI_CAP_DEBUG: u8 = 0x0A;

/// CompactPCI central resource control.
pub const PCI_CAP_CCRC: u8 = 0x0B;

/// PCI Hot-Plug.
pub const PCI_CAP_HOTPLUG: u8 = 0x0C;

/// Bridge subsystem vendor/device ID.
pub const PCI_CAP_SSVID: u8 = 0x0D;

/// AGP 8x.
pub const PCI_CAP_AGP3: u8 = 0x0E;

/// Secure Device.
pub const PCI_CAP_SECURE: u8 = 0x0F;

/// PCI Express.
pub const PCI_CAP_PCIE: u8 = 0x10;

/// MSI-X Capability.
pub const PCI_CAP_MSIX: u8 = 0x11;

/// SATA Data/Index Pair.
pub const PCI_CAP_SATA: u8 = 0x12;

/// Advanced Features.
pub const PCI_CAP_AF: u8 = 0x13;
