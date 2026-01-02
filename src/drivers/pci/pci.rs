//! # Enumeração PCI
//!
//! Escaneia o barramento PCI e detecta dispositivos.
//!
//! ## Offsets do Espaço de Configuração
//!
//! | Offset | Tamanho | Descrição          |
//! |--------|---------|---------------------|
//! | 0x00   | 2       | Vendor ID           |
//! | 0x02   | 2       | Device ID           |
//! | 0x04   | 2       | Command             |
//! | 0x06   | 2       | Status              |
//! | 0x08   | 1       | Revision ID         |
//! | 0x09   | 3       | Class Code          |
//! | 0x0E   | 1       | Header Type         |
//! | 0x10   | 4       | BAR0                |
//! | ...    | ...     | ...                 |

use super::config;
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// Vendor ID inválido (dispositivo não existe)
const VENDOR_INVALID: u16 = 0xFFFF;

/// Vendor ID da Red Hat (VirtIO)
pub const VENDOR_REDHAT: u16 = 0x1AF4;

/// Device ID do VirtIO Block
pub const DEVICE_VIRTIO_BLK: u16 = 0x1001;

/// Device ID do VirtIO Net
pub const DEVICE_VIRTIO_NET: u16 = 0x1000;

/// Device ID do VirtIO GPU
pub const DEVICE_VIRTIO_GPU: u16 = 0x1050;

/// Informações de um dispositivo PCI
#[derive(Debug, Clone)]
pub struct PciDevice {
    /// Número do barramento
    pub bus: u8,
    /// Número do dispositivo
    pub device: u8,
    /// Número da função
    pub function: u8,
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Código de classe
    pub class_code: u8,
    /// Subclasse
    pub subclass: u8,
    /// Interface de programação
    pub prog_if: u8,
    /// Revision ID
    pub revision: u8,
    /// Header type
    pub header_type: u8,
    /// Base Address Registers
    pub bars: [u32; 6],
}

impl PciDevice {
    /// Lê as informações de um dispositivo PCI
    pub fn read(bus: u8, device: u8, function: u8) -> Option<Self> {
        let vendor_id = config::read_config_word(bus, device, function, 0x00);

        // Dispositivo não existe
        if vendor_id == VENDOR_INVALID {
            return None;
        }

        let device_id = config::read_config_word(bus, device, function, 0x02);
        let revision = config::read_config_byte(bus, device, function, 0x08);
        let prog_if = config::read_config_byte(bus, device, function, 0x09);
        let subclass = config::read_config_byte(bus, device, function, 0x0A);
        let class_code = config::read_config_byte(bus, device, function, 0x0B);
        let header_type = config::read_config_byte(bus, device, function, 0x0E);

        // Ler BARs
        let mut bars = [0u32; 6];
        for i in 0..6 {
            bars[i] = config::read_config(bus, device, function, 0x10 + (i as u8 * 4));
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

    /// Verifica se é um dispositivo VirtIO
    pub fn is_virtio(&self) -> bool {
        self.vendor_id == VENDOR_REDHAT
    }

    /// Verifica se é um dispositivo VirtIO Block
    pub fn is_virtio_blk(&self) -> bool {
        self.vendor_id == VENDOR_REDHAT && self.device_id == DEVICE_VIRTIO_BLK
    }

    /// Habilita Bus Mastering (necessário para DMA)
    pub fn enable_bus_master(&self) {
        let command = config::read_config_word(self.bus, self.device, self.function, 0x04);
        // Bit 2 = Bus Master Enable
        config::write_config_word(self.bus, self.device, self.function, 0x04, command | 0x04);
    }

    /// Habilita Memory Space Access
    pub fn enable_memory_space(&self) {
        let command = config::read_config_word(self.bus, self.device, self.function, 0x04);
        // Bit 1 = Memory Space Enable
        config::write_config_word(self.bus, self.device, self.function, 0x04, command | 0x02);
    }

    /// Obtém o endereço base de um BAR (Memory-mapped)
    pub fn bar_address(&self, bar: usize) -> Option<u64> {
        if bar >= 6 {
            return None;
        }

        let value = self.bars[bar];

        // Bit 0 = 0 indica Memory BAR
        if value & 1 != 0 {
            return None; // É I/O BAR
        }

        // Bits 2:1 indicam tipo
        let bar_type = (value >> 1) & 0x3;

        match bar_type {
            0 => {
                // 32-bit address
                Some((value & 0xFFFF_FFF0) as u64)
            }
            2 => {
                // 64-bit address
                if bar + 1 >= 6 {
                    return None;
                }
                let high = self.bars[bar + 1] as u64;
                Some(((high << 32) | (value & 0xFFFF_FFF0) as u64))
            }
            _ => None,
        }
    }
}

/// Lista global de dispositivos PCI detectados
static PCI_DEVICES: Spinlock<Vec<PciDevice>> = Spinlock::new(Vec::new());

/// Escaneia o barramento PCI e detecta todos os dispositivos
pub fn scan() {
    crate::kinfo!("(PCI) Escaneando barramento...");

    let mut devices = PCI_DEVICES.lock();
    devices.clear();

    let mut count = 0;

    // Escanear todos os barramentos (0-255)
    // Na prática, a maioria dos sistemas só usa o barramento 0
    for bus in 0..=255u8 {
        for device in 0..32u8 {
            // Verificar função 0
            if let Some(dev) = PciDevice::read(bus, device, 0) {
                let is_multi = (dev.header_type & 0x80) != 0;

                log_device(&dev);
                devices.push(dev);
                count += 1;

                // Se for multi-função, verificar funções 1-7
                if is_multi {
                    for function in 1..8u8 {
                        if let Some(dev) = PciDevice::read(bus, device, function) {
                            log_device(&dev);
                            devices.push(dev);
                            count += 1;
                        }
                    }
                }
            }
        }

        // Otimização: se o barramento 0 não tiver dispositivos, provavelmente não há mais
        if bus == 0 && count == 0 {
            break;
        }
    }

    crate::kinfo!("(PCI) Dispositivos encontrados:", count as u64);
}

/// Loga informações de um dispositivo
fn log_device(dev: &PciDevice) {
    crate::kdebug!("(PCI) Dispositivo detectado:");
    crate::kdebug!("  Bus:", dev.bus as u64);
    crate::kdebug!("  Device:", dev.device as u64);
    crate::kdebug!("  Function:", dev.function as u64);
    crate::kdebug!("  Vendor:", dev.vendor_id as u64);
    crate::kdebug!("  DeviceID:", dev.device_id as u64);

    if dev.is_virtio_blk() {
        crate::kinfo!("(PCI) VirtIO Block detectado!");
    }
}

/// Procura um dispositivo PCI pelo Vendor/Device ID
pub fn find_device(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    let devices = PCI_DEVICES.lock();
    devices
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
        .cloned()
}

/// Procura um dispositivo VirtIO Block
pub fn find_virtio_blk() -> Option<PciDevice> {
    find_device(VENDOR_REDHAT, DEVICE_VIRTIO_BLK)
}

/// Retorna todos os dispositivos PCI detectados
pub fn all_devices() -> Vec<PciDevice> {
    PCI_DEVICES.lock().clone()
}
