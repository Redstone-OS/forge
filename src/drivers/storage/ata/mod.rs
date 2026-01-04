//! # ATA/IDE Legacy Driver
//!
//! Driver para controladores de disco ATA (PATA/IDE) em modo PIO.
//! Essencial para compatibilidade com BIOS antigos e emuladores.

pub mod io;
pub mod pio;
pub mod ports;

use super::traits::{BlockDevice, BlockError};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct AtaDriver;

impl Driver for AtaDriver {
    fn name(&self) -> &'static str {
        "Legacy ATA/IDE Driver (PIO)"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Detectar drives no boot (Master/Slave)
        // 1. Verificar Status na porta 0x1F7
        // 2. Enviar comando IDENTIFY (0xEC)
        crate::kinfo!("(Storage/ATA) Buscando dispositivos IDE...");

        // Simulação: Inicializa o drive se encontrado
        if let Some(drive) = AtaDisk::new(0, false) {
            crate::kinfo!("(Storage/ATA) Primary Master detectado.");
            // TODO: Registrar este 'drive' no sistema de arquivos
        }

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub struct AtaDisk {
    bus: u8, // 0=Primary, 1=Secondary
    slave: bool,
    sectors: u64,
}

impl AtaDisk {
    pub fn new(bus: u8, slave: bool) -> Option<Self> {
        // STUB: Aqui iria a lógica real de IDENTIFY via ports::...
        Some(Self {
            bus,
            slave,
            sectors: 1024 * 1024, // Fake 512MB
        })
    }
}

impl BlockDevice for AtaDisk {
    fn name(&self) -> &str {
        if self.bus == 0 {
            "hda"
        } else {
            "hdb"
        }
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        self.sectors
    }

    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        // STUB: Chamar pio::read(lba, buf)
        // Por enquanto retornamos zeros
        buf.fill(0);
        Ok(())
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // STUB: Chamar pio::write(lba, buf)
        Ok(())
    }
}
