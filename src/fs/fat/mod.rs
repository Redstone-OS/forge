//! # Driver de Sistema de Arquivos FAT
//!
//! Suporta FAT16 e FAT32 para leitura de arquivos do disco.
//!
//! ## Arquitetura FAT
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  Boot Sector (BPB) │  FAT Table  │  Root Dir │ Data  │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Estrutura do Módulo
//!
//! - `bpb.rs` - Parser do BIOS Parameter Block (boot sector)
//! - `dir.rs` - Leitura de entradas de diretório
//! - `file.rs` - Operações de leitura de arquivos

pub mod bpb;
pub mod dir;
pub mod file;

use crate::drivers::block::{BlockDevice, BlockError};
use crate::fs::vfs::inode::FsError;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Instância do filesystem FAT
pub struct FatFs {
    /// Dispositivo de bloco subjacente
    device: Arc<dyn BlockDevice>,
    /// Informações do BPB
    bpb: bpb::Bpb,
    /// Tipo de FAT
    fat_type: FatType,
}

/// Tipo de FAT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

impl FatFs {
    /// Monta um filesystem FAT a partir de um dispositivo de bloco
    pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Self, FsError> {
        crate::kinfo!("(FAT) Montando filesystem...");

        // Ler boot sector
        let mut boot_sector = [0u8; 512];
        device
            .read_block(0, &mut boot_sector)
            .map_err(|_| FsError::IoError)?;

        // Parsear BPB
        let bpb = bpb::Bpb::parse(&boot_sector).ok_or(FsError::IoError)?;

        // Determinar tipo de FAT
        let fat_type = bpb.fat_type();

        crate::kinfo!("(FAT) Tipo detectado:", fat_type as u64);
        crate::kinfo!("(FAT) Bytes por setor:", bpb.bytes_per_sector as u64);
        crate::kinfo!("(FAT) Setores por cluster:", bpb.sectors_per_cluster as u64);

        Ok(Self {
            device,
            bpb,
            fat_type,
        })
    }

    /// Lê um cluster inteiro para um buffer
    pub fn read_cluster(&self, cluster: u32, buf: &mut [u8]) -> Result<usize, FsError> {
        let cluster_size = self.bpb.cluster_size();
        if buf.len() < cluster_size {
            return Err(FsError::IoError);
        }

        let first_sector = self.bpb.cluster_to_sector(cluster);
        let sectors_per_cluster = self.bpb.sectors_per_cluster as u64;

        for i in 0..sectors_per_cluster {
            let sector = first_sector + i;
            let offset = (i as usize) * 512;
            self.device
                .read_block(sector, &mut buf[offset..offset + 512])
                .map_err(|_| FsError::IoError)?;
        }

        Ok(cluster_size)
    }

    /// Obtém o próximo cluster na cadeia da tabela FAT
    pub fn next_cluster(&self, cluster: u32) -> Option<u32> {
        let fat_offset = match self.fat_type {
            FatType::Fat12 => (cluster + (cluster / 2)) as usize,
            FatType::Fat16 => (cluster * 2) as usize,
            FatType::Fat32 => (cluster * 4) as usize,
        };

        let fat_sector = self.bpb.reserved_sectors as u64 + (fat_offset / 512) as u64;
        let entry_offset = fat_offset % 512;

        let mut sector_buf = [0u8; 512];
        self.device.read_block(fat_sector, &mut sector_buf).ok()?;

        let next = match self.fat_type {
            FatType::Fat12 => {
                let val = u16::from_le_bytes([
                    sector_buf[entry_offset],
                    sector_buf.get(entry_offset + 1).copied().unwrap_or(0),
                ]);
                if cluster & 1 != 0 {
                    (val >> 4) as u32
                } else {
                    (val & 0x0FFF) as u32
                }
            }
            FatType::Fat16 => {
                u16::from_le_bytes([sector_buf[entry_offset], sector_buf[entry_offset + 1]]) as u32
            }
            FatType::Fat32 => {
                u32::from_le_bytes([
                    sector_buf[entry_offset],
                    sector_buf[entry_offset + 1],
                    sector_buf[entry_offset + 2],
                    sector_buf[entry_offset + 3],
                ]) & 0x0FFFFFFF
            }
        };

        // Verificar fim da cadeia
        let eoc = match self.fat_type {
            FatType::Fat12 => next >= 0x0FF8,
            FatType::Fat16 => next >= 0xFFF8,
            FatType::Fat32 => next >= 0x0FFFFFF8,
        };

        if eoc || next < 2 {
            None
        } else {
            Some(next)
        }
    }
}

/// Inicializa o módulo FAT
pub fn init() {
    crate::kinfo!("(FAT) Módulo inicializado");
}
