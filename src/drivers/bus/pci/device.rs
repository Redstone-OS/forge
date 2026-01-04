//! # PCI Device Representation
//!
//! Estruturas e métodos para interagir com dispositivos PCI individuais.

use super::access;

/// Vendor ID inválido (dispositivo não existe)
const VENDOR_INVALID: u16 = 0xFFFF;

/// Estrutura técnica de um dispositivo PCI no barramento
#[derive(Debug, Clone, Copy)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub bars: [u32; 6],
}

impl PciDeviceInfo {
    /// Lê as informações de configuração de um SLOT PCI
    pub fn read(bus: u8, device: u8, function: u8) -> Option<Self> {
        let vendor_id = access::read_config_word(bus, device, function, 0x00);

        if vendor_id == VENDOR_INVALID {
            return None;
        }

        let device_id = access::read_config_word(bus, device, function, 0x02);
        let revision = access::read_config_byte(bus, device, function, 0x08);
        let prog_if = access::read_config_byte(bus, device, function, 0x09);
        let subclass = access::read_config_byte(bus, device, function, 0x0A);
        let class_code = access::read_config_byte(bus, device, function, 0x0B);
        let header_type = access::read_config_byte(bus, device, function, 0x0E);

        let mut bars = [0u32; 6];
        for i in 0..6 {
            bars[i] = access::read_config(bus, device, function, 0x10 + (i as u8 * 4));
        }

        Some(Self {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            revision,
            header_type,
            bars,
        })
    }

    /// Habilita Bus Mastering (necessário para DMA)
    pub fn enable_bus_master(&self) {
        let command = access::read_config_word(self.bus, self.device, self.function, 0x04);
        access::write_config_word(self.bus, self.device, self.function, 0x04, command | 0x04);
    }

    /// Habilita Memory Space Enable
    pub fn enable_memory_space(&self) {
        let command = access::read_config_word(self.bus, self.device, self.function, 0x04);
        access::write_config_word(self.bus, self.device, self.function, 0x04, command | 0x02);
    }

    /// Obtém o endereço base de um BAR (Memory-mapped)
    pub fn get_bar_address(&self, bar_index: usize) -> Option<u64> {
        if bar_index >= 6 {
            return None;
        }

        let value = self.bars[bar_index];
        if value & 1 != 0 {
            return None;
        } // I/O BAR não suportado aqui

        let bar_type = (value >> 1) & 0x3;
        match bar_type {
            0 => Some((value & 0xFFFF_FFF0) as u64), // 32-bit
            2 => {
                // 64-bit
                if bar_index + 1 >= 6 {
                    return None;
                }
                let high = self.bars[bar_index + 1] as u64;
                Some((high << 32) | (value & 0xFFFF_FFF0) as u64)
            }
            _ => None,
        }
    }
}
