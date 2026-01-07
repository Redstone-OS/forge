//! # ATA/IDE Driver
//!
//! Driver para discos PATA/IDE legados usando PIO mode.
//! Importante para hardware antigo e QEMU default.
//!
//! ## Portas I/O:
//! - Primary: 0x1F0-0x1F7, Control: 0x3F6
//! - Secondary: 0x170-0x177, Control: 0x376

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use core::arch::asm;

// =============================================================================
// ATA I/O PORTS
// =============================================================================

const ATA_PRIMARY_BASE: u16 = 0x1F0;
const ATA_PRIMARY_CTRL: u16 = 0x3F6;
const ATA_SECONDARY_BASE: u16 = 0x170;
#[allow(unused)]
const ATA_SECONDARY_CTRL: u16 = 0x376;

// Offsets from base port
const ATA_REG_DATA: u16 = 0;
#[allow(unused)]
const ATA_REG_ERROR: u16 = 1;
const ATA_REG_SECCOUNT: u16 = 2;
const ATA_REG_LBA0: u16 = 3;
const ATA_REG_LBA1: u16 = 4;
const ATA_REG_LBA2: u16 = 5;
const ATA_REG_HDDEVSEL: u16 = 6;
const ATA_REG_COMMAND: u16 = 7;
const ATA_REG_STATUS: u16 = 7;

// Status bits
const ATA_SR_BSY: u8 = 0x80; // Busy
#[allow(unused)]
const ATA_SR_DRDY: u8 = 0x40; // Drive ready
const ATA_SR_DRQ: u8 = 0x08; // Data request ready
const ATA_SR_ERR: u8 = 0x01; // Error

// Commands
const ATA_CMD_READ_PIO: u8 = 0x20;
#[allow(unused)]
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

// =============================================================================
// PORT I/O
// =============================================================================

#[inline]
fn port_read_u8(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    }
    value
}

#[inline]
fn port_write_u8(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }
}

#[inline]
fn port_read_u16(port: u16) -> u16 {
    let value: u16;
    unsafe {
        asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
    }
    value
}

// =============================================================================
// DRIVER RDS
// =============================================================================

/// Driver ATA para o RDS.
pub struct AtaDriver;

impl Driver for AtaDriver {
    fn name(&self) -> &'static str {
        "ata"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // ATA é detectado via portas I/O, não PCI tradicional
        // Aceita dispositivos de classe IDE (0x01, 0x01)
        if dev.class_code != 0x01 || dev.subclass_code != 0x01 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(ATA) Controlador IDE: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // Tentar detectar discos no canal primário
        if let Some(disk) = AtaDisk::detect(0, 0) {
            let capacity_mb = disk.capacity() / (1024 * 1024);
            crate::kinfo!(
                "(ATA) Disco encontrado: {} ({}MB)",
                disk.name(),
                capacity_mb
            );
            crate::drivers::storage::register_device(Arc::new(disk));
        }

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(ATA) Driver removido");
        Ok(())
    }
}

// =============================================================================
// ATA DISK
// =============================================================================

struct AtaDiskState {
    enabled: bool,
    capacity_sectors: u64,
    stats: StorageStats,
}

/// Dispositivo ATA/IDE.
pub struct AtaDisk {
    channel: u8, // 0 = primary, 1 = secondary
    drive: u8,   // 0 = master, 1 = slave
    base: u16,
    ctrl: u16,
    state: Spinlock<AtaDiskState>,
}

impl AtaDisk {
    /// Detecta um disco ATA no canal/drive especificado.
    pub fn detect(channel: u8, drive: u8) -> Option<Self> {
        let base = if channel == 0 {
            ATA_PRIMARY_BASE
        } else {
            ATA_SECONDARY_BASE
        };
        let ctrl = if channel == 0 {
            ATA_PRIMARY_CTRL
        } else {
            ATA_SECONDARY_CTRL
        };

        // Selecionar drive
        let drive_sel = 0xA0 | ((drive & 1) << 4);
        port_write_u8(base + ATA_REG_HDDEVSEL, drive_sel);

        // Pequeno delay
        for _ in 0..15 {
            let _ = port_read_u8(ctrl);
        }

        // Zerar registradores LBA
        port_write_u8(base + ATA_REG_SECCOUNT, 0);
        port_write_u8(base + ATA_REG_LBA0, 0);
        port_write_u8(base + ATA_REG_LBA1, 0);
        port_write_u8(base + ATA_REG_LBA2, 0);

        // Enviar IDENTIFY
        port_write_u8(base + ATA_REG_COMMAND, ATA_CMD_IDENTIFY);

        // Ler status
        let status = port_read_u8(base + ATA_REG_STATUS);
        if status == 0 {
            return None; // Sem disco
        }

        // Esperar BSY limpar
        let mut timeout = 100_000u32;
        while port_read_u8(base + ATA_REG_STATUS) & ATA_SR_BSY != 0 {
            timeout = timeout.saturating_sub(1);
            if timeout == 0 {
                return None;
            }
        }

        // Verificar se é ATAPI (não suportado)
        let lba1 = port_read_u8(base + ATA_REG_LBA1);
        let lba2 = port_read_u8(base + ATA_REG_LBA2);
        if lba1 != 0 || lba2 != 0 {
            return None; // ATAPI ou SATA
        }

        // Esperar DRQ
        timeout = 100_000;
        loop {
            let st = port_read_u8(base + ATA_REG_STATUS);
            if st & ATA_SR_ERR != 0 {
                return None;
            }
            if st & ATA_SR_DRQ != 0 {
                break;
            }
            timeout = timeout.saturating_sub(1);
            if timeout == 0 {
                return None;
            }
        }

        // Ler 256 words de dados IDENTIFY
        let mut identify = [0u16; 256];
        for word in identify.iter_mut() {
            *word = port_read_u16(base + ATA_REG_DATA);
        }

        // Extrair capacidade (words 60-61 para LBA28)
        let capacity_sectors = (identify[61] as u64) << 16 | (identify[60] as u64);

        if capacity_sectors == 0 {
            return None;
        }

        Some(Self {
            channel,
            drive,
            base,
            ctrl,
            state: Spinlock::new(AtaDiskState {
                enabled: true,
                capacity_sectors,
                stats: StorageStats::default(),
            }),
        })
    }

    /// Espera até que o disco não esteja ocupado.
    fn wait_not_busy(&self) -> Result<(), BlockError> {
        let mut timeout = 1_000_000u32;
        while port_read_u8(self.base + ATA_REG_STATUS) & ATA_SR_BSY != 0 {
            timeout = timeout.saturating_sub(1);
            if timeout == 0 {
                return Err(BlockError::Timeout);
            }
            core::hint::spin_loop();
        }
        Ok(())
    }

    /// Espera por DRQ.
    fn wait_drq(&self) -> Result<(), BlockError> {
        let mut timeout = 1_000_000u32;
        loop {
            let st = port_read_u8(self.base + ATA_REG_STATUS);
            if st & ATA_SR_ERR != 0 {
                return Err(BlockError::IoError);
            }
            if st & ATA_SR_DRQ != 0 {
                return Ok(());
            }
            timeout = timeout.saturating_sub(1);
            if timeout == 0 {
                return Err(BlockError::Timeout);
            }
            core::hint::spin_loop();
        }
    }
}

impl BlockDevice for AtaDisk {
    fn name(&self) -> &str {
        match (self.channel, self.drive) {
            (0, 0) => "hda",
            (0, 1) => "hdb",
            (1, 0) => "hdc",
            (1, 1) => "hdd",
            _ => "hdX",
        }
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("ATA/IDE Disk"),
            device_type: Some(StorageType::Hdd),
            interface: Some(StorageInterface::Ata),
            ..Default::default()
        }
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        self.state.lock().capacity_sectors
    }

    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if buf.len() < 512 {
            return Err(BlockError::InvalidBuffer);
        }

        let state = self.state.lock();
        if !state.enabled {
            return Err(BlockError::NotReady);
        }
        if lba >= state.capacity_sectors {
            return Err(BlockError::InvalidLba);
        }
        drop(state);

        self.wait_not_busy()?;

        // Selecionar drive + LBA bits 24-27
        let drive_sel = 0xE0 | ((self.drive & 1) << 4) | ((lba >> 24) as u8 & 0x0F);
        port_write_u8(self.base + ATA_REG_HDDEVSEL, drive_sel);

        // Delay
        for _ in 0..5 {
            let _ = port_read_u8(self.ctrl);
        }

        // Configurar LBA e sector count
        port_write_u8(self.base + ATA_REG_SECCOUNT, 1);
        port_write_u8(self.base + ATA_REG_LBA0, (lba & 0xFF) as u8);
        port_write_u8(self.base + ATA_REG_LBA1, ((lba >> 8) & 0xFF) as u8);
        port_write_u8(self.base + ATA_REG_LBA2, ((lba >> 16) & 0xFF) as u8);

        // Enviar comando READ PIO
        port_write_u8(self.base + ATA_REG_COMMAND, ATA_CMD_READ_PIO);

        self.wait_drq()?;

        // Ler 256 words (512 bytes)
        let words = buf.as_mut_ptr() as *mut u16;
        for i in 0..256 {
            unsafe {
                *words.add(i) = port_read_u16(self.base + ATA_REG_DATA);
            }
        }

        // Atualizar stats
        let mut state = self.state.lock();
        state.stats.blocks_read += 1;
        state.stats.bytes_read += 512;

        Ok(())
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: Implementar escrita PIO
        Err(BlockError::NotReady)
    }

    /// Leitura otimizada multi-setor (até 16 setores por comando)
    fn read_blocks(&self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        if buf.len() < count * 512 {
            return Err(BlockError::InvalidBuffer);
        }
        if count == 0 {
            return Ok(());
        }

        // Limitar a 16 setores por comando para segurança
        let max_sectors = 16usize;
        let mut offset = 0;
        let mut current_lba = lba;
        let mut remaining = count;

        while remaining > 0 {
            let batch = remaining.min(max_sectors);

            {
                let state = self.state.lock();
                if !state.enabled {
                    return Err(BlockError::NotReady);
                }
                if current_lba + batch as u64 > state.capacity_sectors {
                    return Err(BlockError::InvalidLba);
                }
            }

            self.wait_not_busy()?;

            // Selecionar drive + LBA bits 24-27
            let drive_sel = 0xE0 | ((self.drive & 1) << 4) | ((current_lba >> 24) as u8 & 0x0F);
            port_write_u8(self.base + ATA_REG_HDDEVSEL, drive_sel);

            // Delay mínimo
            for _ in 0..4 {
                let _ = port_read_u8(self.ctrl);
            }

            // Configurar LBA e sector count
            port_write_u8(self.base + ATA_REG_SECCOUNT, batch as u8);
            port_write_u8(self.base + ATA_REG_LBA0, (current_lba & 0xFF) as u8);
            port_write_u8(self.base + ATA_REG_LBA1, ((current_lba >> 8) & 0xFF) as u8);
            port_write_u8(self.base + ATA_REG_LBA2, ((current_lba >> 16) & 0xFF) as u8);

            // Enviar comando READ PIO
            port_write_u8(self.base + ATA_REG_COMMAND, ATA_CMD_READ_PIO);

            // Ler cada setor
            for s in 0..batch {
                self.wait_drq()?;

                // Ler 256 words (512 bytes)
                let sector_offset = offset + s * 512;
                let words = buf[sector_offset..].as_mut_ptr() as *mut u16;
                for i in 0..256 {
                    unsafe {
                        *words.add(i) = port_read_u16(self.base + ATA_REG_DATA);
                    }
                }
            }

            // Atualizar stats
            let mut state = self.state.lock();
            state.stats.blocks_read += batch as u64;
            state.stats.bytes_read += (batch * 512) as u64;
            drop(state);

            offset += batch * 512;
            current_lba += batch as u64;
            remaining -= batch;
        }

        Ok(())
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

/// Registra o driver ATA.
pub fn init() {
    crate::kinfo!("(ATA) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(AtaDriver));
}
