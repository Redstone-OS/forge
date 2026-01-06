//! # Intel GPU PCI Driver
//!
//! Detecção e inicialização do GPU Intel via PCI.
//! Registra o driver no RDS para matching automático.

use crate::drivers::base::bus::BusAddress;
use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::config;
use crate::drivers::bus::pci::{self, PciAddress};
use crate::rmm::addr::VirtAddr;
use alloc::sync::Arc;

use super::device::IntelDevice;
use super::hw::{gen9, Mmio};

// =============================================================================
// CONSTANTES
// =============================================================================

/// Intel Vendor ID
const INTEL_VENDOR_ID: u16 = 0x8086;

// =============================================================================
// BAR PARSING
// =============================================================================

/// Informação de um BAR lido
// Todo: Revisar
#[allow(unused)]
struct BarInfo {
    address: u64,
    size: u64,
    is_64bit: bool,
    is_mmio: bool,
}

/// Lê e parseia um BAR de 32/64 bits
fn read_bar(pci_addr: PciAddress, bar_offset: u8) -> BarInfo {
    // Lê o valor atual do BAR
    let bar_low = pci::read_config(pci_addr, bar_offset);

    // Verifica se é I/O ou MMIO
    let is_mmio = (bar_low & 0x01) == 0;

    if !is_mmio {
        // I/O BAR - não suportado para GPU
        return BarInfo {
            address: (bar_low & !0x03) as u64,
            size: 0,
            is_64bit: false,
            is_mmio: false,
        };
    }

    // MMIO BAR - verifica se é 32 ou 64 bits
    let bar_type = (bar_low >> 1) & 0x03;
    let is_64bit = bar_type == 2;

    let address: u64;
    let size: u64;

    if is_64bit {
        // BAR de 64 bits - lê o próximo registro também
        let bar_high = pci::read_config(pci_addr, bar_offset + 4);
        address = ((bar_high as u64) << 32) | ((bar_low & !0x0F) as u64);

        // Para obter o tamanho, precisaríamos escrever 0xFFFFFFFF e ler de volta
        // Por simplicidade, assumimos tamanhos típicos para Intel GPU
        size = if bar_offset == config::PCI_BAR0 {
            16 * 1024 * 1024 // 16MB MMIO típico
        } else {
            256 * 1024 * 1024 // 256MB Aperture típico
        };
    } else {
        // BAR de 32 bits
        address = (bar_low & !0x0F) as u64;
        size = 4 * 1024 * 1024; // 4MB típico para 32-bit
    }

    BarInfo {
        address,
        size,
        is_64bit,
        is_mmio: true,
    }
}

// =============================================================================
// DRIVER RDS
// =============================================================================

/// Driver Intel GPU para o RDS.
pub struct IntelGpuDriver;

impl Driver for IntelGpuDriver {
    fn name(&self) -> &'static str {
        "Intel Integrated Graphics"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // Verificar vendor Intel
        if dev.vendor_id != INTEL_VENDOR_ID {
            return Err(DriverError::NotSupported);
        }

        // Verificar se é GPU Gen9 LP suportada
        if !gen9::is_gen9_lp(dev.device_id) {
            return Err(DriverError::NotSupported);
        }

        let device_name = gen9::device_name(dev.device_id);
        crate::kinfo!("(Intel GPU) Detectado:", device_name);

        // Extrair endereço PCI
        let pci_addr = match dev.bus_address {
            BusAddress::Pci {
                segment,
                bus,
                device,
                function,
            } => PciAddress::new(segment, bus, device, function),
            _ => return Err(DriverError::NotSupported),
        };

        // Habilitar Bus Master e Memory Space
        pci::enable_bus_master(pci_addr);

        // Ler BARs
        let mmio_bar = read_bar(pci_addr, config::PCI_BAR0);
        let aperture_bar = read_bar(pci_addr, config::PCI_BAR2);

        if mmio_bar.address == 0 {
            crate::kerror!("(Intel GPU) BAR0 (MMIO) inválido");
            return Err(DriverError::HardwareFault);
        }

        crate::kdebug!("(Intel GPU) MMIO BAR:", mmio_bar.address);
        crate::kdebug!("(Intel GPU) MMIO Size:", mmio_bar.size);

        if aperture_bar.address != 0 {
            crate::kdebug!("(Intel GPU) Aperture BAR:", aperture_bar.address);
            crate::kdebug!("(Intel GPU) Aperture Size:", aperture_bar.size);
        }

        // Mapear MMIO para espaço virtual
        let mmio_virt = crate::rmm::virt::hhdm::phys_to_virt(mmio_bar.address);

        let mmio = unsafe { Mmio::new(VirtAddr::new(mmio_virt as u64), mmio_bar.size as usize) };

        // Criar dispositivo Intel
        match IntelDevice::new(dev.device_id, mmio, aperture_bar.address, pci_addr) {
            Some(intel_dev) => {
                crate::kinfo!("(Intel GPU) Dispositivo inicializado!");
                crate::drivers::display::register_device(Arc::new(intel_dev));
                Ok(())
            }
            None => {
                crate::kerror!("(Intel GPU) Falha ao inicializar dispositivo");
                Err(DriverError::HardwareFault)
            }
        }
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::display::unregister_device("Intel HD Graphics");
        Ok(())
    }
}
