//! # BIOS Parameter Block (BPB)
//!
//! Parser do boot sector para extrair metadados do filesystem FAT.
//!
//! ## Estrutura do Boot Sector
//!
//! | Offset | Tamanho | Descrição                    |
//! |--------|---------|------------------------------|
//! | 0x00   | 3       | Jump instruction             |
//! | 0x03   | 8       | OEM Name                     |
//! | 0x0B   | 2       | Bytes por setor              |
//! | 0x0D   | 1       | Setores por cluster          |
//! | 0x0E   | 2       | Setores reservados           |
//! | 0x10   | 1       | Número de FATs               |
//! | 0x11   | 2       | Entradas no root (FAT12/16)  |
//! | ...    | ...     | ...                          |

use super::FatType;

/// BIOS Parameter Block
#[derive(Debug, Clone)]
pub struct Bpb {
    /// Bytes por setor (geralmente 512)
    pub bytes_per_sector: u16,
    /// Setores por cluster
    pub sectors_per_cluster: u8,
    /// Setores reservados antes da FAT
    pub reserved_sectors: u16,
    /// Número de FATs (geralmente 2)
    pub num_fats: u8,
    /// Entradas no diretório raiz (FAT12/16 apenas)
    pub root_entry_count: u16,
    /// Total de setores (16-bit, 0 se usar 32-bit)
    pub total_sectors_16: u16,
    /// Setores por FAT (FAT12/16)
    pub sectors_per_fat_16: u16,
    /// Total de setores (32-bit)
    pub total_sectors_32: u32,
    /// Setores por FAT (FAT32)
    pub sectors_per_fat_32: u32,
    /// Cluster do diretório raiz (FAT32)
    pub root_cluster: u32,
}

impl Bpb {
    /// Faz o parse do BPB a partir dos bytes do boot sector
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 512 {
            return None;
        }

        // Verificar assinatura de boot válida
        if data[510] != 0x55 || data[511] != 0xAA {
            return None;
        }

        let bytes_per_sector = u16::from_le_bytes([data[11], data[12]]);
        let sectors_per_cluster = data[13];
        let reserved_sectors = u16::from_le_bytes([data[14], data[15]]);
        let num_fats = data[16];
        let root_entry_count = u16::from_le_bytes([data[17], data[18]]);
        let total_sectors_16 = u16::from_le_bytes([data[19], data[20]]);
        let sectors_per_fat_16 = u16::from_le_bytes([data[22], data[23]]);
        let total_sectors_32 = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);

        // Campos específicos do FAT32
        let sectors_per_fat_32 = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let root_cluster = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);

        Some(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            root_entry_count,
            total_sectors_16,
            sectors_per_fat_16,
            total_sectors_32,
            sectors_per_fat_32,
            root_cluster,
        })
    }

    /// Determina o tipo de FAT baseado na contagem de clusters
    pub fn fat_type(&self) -> FatType {
        let root_dir_sectors = ((self.root_entry_count as u32 * 32)
            + (self.bytes_per_sector as u32 - 1))
            / self.bytes_per_sector as u32;

        let fat_size = if self.sectors_per_fat_16 != 0 {
            self.sectors_per_fat_16 as u32
        } else {
            self.sectors_per_fat_32
        };

        let total_sectors = if self.total_sectors_16 != 0 {
            self.total_sectors_16 as u32
        } else {
            self.total_sectors_32
        };

        let data_sectors = total_sectors
            - (self.reserved_sectors as u32 + (self.num_fats as u32 * fat_size) + root_dir_sectors);

        let count_of_clusters = data_sectors / self.sectors_per_cluster as u32;

        if count_of_clusters < 4085 {
            FatType::Fat12
        } else if count_of_clusters < 65525 {
            FatType::Fat16
        } else {
            FatType::Fat32
        }
    }

    /// Retorna o tamanho do cluster em bytes
    pub fn cluster_size(&self) -> usize {
        self.bytes_per_sector as usize * self.sectors_per_cluster as usize
    }

    /// Retorna o primeiro setor de dados
    pub fn first_data_sector(&self) -> u64 {
        let root_dir_sectors = ((self.root_entry_count as u32 * 32)
            + (self.bytes_per_sector as u32 - 1))
            / self.bytes_per_sector as u32;

        let fat_size = if self.sectors_per_fat_16 != 0 {
            self.sectors_per_fat_16 as u32
        } else {
            self.sectors_per_fat_32
        };

        (self.reserved_sectors as u32 + (self.num_fats as u32 * fat_size) + root_dir_sectors) as u64
    }

    /// Converte número de cluster para número de setor
    pub fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let first_data_sector = self.first_data_sector();
        first_data_sector + ((cluster - 2) as u64 * self.sectors_per_cluster as u64)
    }

    /// Retorna setores por FAT
    pub fn sectors_per_fat(&self) -> u32 {
        if self.sectors_per_fat_16 != 0 {
            self.sectors_per_fat_16 as u32
        } else {
            self.sectors_per_fat_32
        }
    }

    /// Retorna o primeiro setor do diretório raiz (FAT12/16 apenas)
    pub fn root_dir_sector(&self) -> u64 {
        (self.reserved_sectors as u64) + (self.num_fats as u64 * self.sectors_per_fat() as u64)
    }
}
