//! # FatFs - Struct Principal do Filesystem FAT
//!
//! Versão Stack-Safe e Otimizada.

use super::bpb::Bpb;
use super::dir::DirEntry;
use super::PublicDirEntry;
use crate::drivers::block::BlockDevice;
use crate::fs::vfs::inode::FsError;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// TIPOS
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

// =============================================================================
// FATFS
// =============================================================================

pub struct FatFs {
    device: Arc<dyn BlockDevice>,
    bpb: Bpb,
    fat_type: FatType,
    partition_offset: u64,
}

impl FatFs {
    pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Self, FsError> {
        let mut sector0 = [0u8; 512];
        device
            .read_block(0, &mut sector0)
            .map_err(|_| FsError::IoError)?;

        let partition_start = if sector0[0] == 0xEB || sector0[0] == 0xE9 {
            0u64
        } else if sector0[510] == 0x55 && sector0[511] == 0xAA {
            let part_entry = &sector0[0x1BE..0x1BE + 16];
            u32::from_le_bytes([part_entry[8], part_entry[9], part_entry[10], part_entry[11]])
                as u64
        } else {
            return Err(FsError::InvalidFormat);
        };

        let mut boot_sector = [0u8; 512];
        device
            .read_block(partition_start, &mut boot_sector)
            .map_err(|_| FsError::IoError)?;

        let bpb = Bpb::parse(&boot_sector).ok_or(FsError::InvalidFormat)?;
        if bpb.bytes_per_sector == 0 || bpb.sectors_per_cluster == 0 {
            return Err(FsError::InvalidFormat);
        }

        let fat_type = bpb.fat_type();
        crate::kinfo!("(FAT) Montado. Tipo:", fat_type as u64);
        crate::kinfo!("(FAT) Sectors per FAT:", bpb.sectors_per_fat() as u64);
        crate::kinfo!("(FAT) Reserved sectors:", bpb.reserved_sectors as u64);
        crate::kinfo!("(FAT) Root entries:", bpb.root_entry_count as u64);

        Ok(Self {
            device,
            bpb,
            fat_type,
            partition_offset: partition_start,
        })
    }

    // --- Helpers de Cache de Setor para evitar alocações no stack ---

    fn read_sector(&self, sector: u64, buf: &mut [u8; 512]) -> Result<(), FsError> {
        self.device
            .read_block(sector, buf)
            .map_err(|_| FsError::IoError)
    }

    /// Lê um cluster inteiro para um buffer (usado por file.rs)
    pub fn read_cluster(&self, cluster: u32, buf: &mut [u8]) -> Result<usize, FsError> {
        let cluster_size = self.bpb.cluster_size();
        if buf.len() < cluster_size {
            return Err(FsError::IoError);
        }

        let first_sector = self.bpb.cluster_to_sector(cluster) + self.partition_offset;
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

    pub fn next_cluster(&self, cluster: u32) -> Option<u32> {
        let mut sector_buf = [0u8; 512];
        let fat_offset = match self.fat_type {
            FatType::Fat12 => (cluster + (cluster / 2)) as usize,
            FatType::Fat16 => (cluster * 2) as usize,
            FatType::Fat32 => (cluster * 4) as usize,
        };

        let fat_sector =
            self.partition_offset + self.bpb.reserved_sectors as u64 + (fat_offset / 512) as u64;
        let entry_offset = fat_offset % 512;

        if self.read_sector(fat_sector, &mut sector_buf).is_err() {
            return None;
        }

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

        let is_eoc = match self.fat_type {
            FatType::Fat12 => next >= 0x0FF8,
            FatType::Fat16 => next >= 0xFFF8,
            FatType::Fat32 => next >= 0x0FFFFFF8,
        };

        if is_eoc || next < 2 {
            None
        } else {
            Some(next)
        }
    }

    // =========================================================================
    // LEITURA DE ARQUIVOS (Otimizada)
    // =========================================================================

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return None;
        }

        crate::ktrace!("(FAT) read_file buscando path:", path);

        let root_cluster = if self.fat_type == FatType::Fat32 {
            self.bpb.root_cluster
        } else {
            0
        };
        let mut current_cluster = root_cluster;

        // Iterar sobre componentes sem alocar Vec
        let mut components = path.split('/').filter(|s| !s.is_empty()).peekable();

        while let Some(component) = components.next() {
            let is_last = components.peek().is_none();

            if let Some(entry) = self.find_entry(current_cluster, component) {
                crate::ktrace!("(FAT) componente encontrado:", component);
                if is_last {
                    if entry.is_directory() {
                        return None;
                    }
                    return self.read_file_data(entry.first_cluster(), entry.size);
                } else {
                    if !entry.is_directory() {
                        return None;
                    }
                    current_cluster = entry.first_cluster();
                }
            } else {
                return None;
            }
        }
        None
    }

    fn read_file_data(&self, first_cluster: u32, size: u32) -> Option<Vec<u8>> {
        let mut data = Vec::with_capacity(size as usize);
        let sectors_per_cluster = self.bpb.sectors_per_cluster as u64;
        let mut sector_buf = [0u8; 512];
        let mut remaining = size as usize;
        let mut cluster = first_cluster;

        loop {
            let first_sector = self.bpb.cluster_to_sector(cluster) + self.partition_offset;
            for i in 0..sectors_per_cluster {
                if self.read_sector(first_sector + i, &mut sector_buf).is_err() {
                    return None;
                }
                let to_copy = remaining.min(512);
                data.extend_from_slice(&sector_buf[..to_copy]);
                remaining -= to_copy;
                if remaining == 0 {
                    break;
                }
            }

            if remaining == 0 {
                break;
            }
            match self.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }
        Some(data)
    }

    fn find_entry(&self, dir_cluster: u32, name: &str) -> Option<DirEntry> {
        crate::ktrace!("(FAT) find_entry buscando:", name);
        if dir_cluster == 0 && self.fat_type != FatType::Fat32 {
            return self.find_in_root_dir(name);
        }

        let sectors_per_cluster = self.bpb.sectors_per_cluster as u64;
        let mut sector_buf = [0u8; 512];
        let mut cluster = dir_cluster;

        loop {
            let first_sector = self.bpb.cluster_to_sector(cluster) + self.partition_offset;
            for s in 0..sectors_per_cluster {
                if self.read_sector(first_sector + s, &mut sector_buf).is_err() {
                    return None;
                }
                for i in 0..16 {
                    let entry_data = &sector_buf[i * 32..(i + 1) * 32];

                    // Fim do diretório
                    if entry_data[0] == 0x00 {
                        return None;
                    }
                    // Deletado
                    if entry_data[0] == 0xE5 {
                        continue;
                    }

                    if let Some(entry) = DirEntry::parse(entry_data) {
                        if Self::names_equal(&entry.name, name) {
                            return Some(entry);
                        } else {
                            crate::ktrace!("(FAT) entrada no disco:", entry.name.as_str());
                        }
                    }
                }
            }
            match self.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }
        None
    }

    fn find_in_root_dir(&self, name: &str) -> Option<DirEntry> {
        let root_dir_sectors = ((self.bpb.root_entry_count as u32 * 32) + 511) / 512;
        let first_root_sector = self.partition_offset
            + self.bpb.reserved_sectors as u64
            + (self.bpb.num_fats as u64 * self.bpb.sectors_per_fat() as u64);
        let mut sector_buf = [0u8; 512];

        for i in 0..root_dir_sectors as u64 {
            if self
                .read_sector(first_root_sector + i, &mut sector_buf)
                .is_err()
            {
                continue;
            }
            for j in 0..16 {
                if let Some(entry) = DirEntry::parse(&sector_buf[j * 32..(j + 1) * 32]) {
                    if Self::names_equal(&entry.name, name) {
                        return Some(entry);
                    } else {
                        // Log agressivo para ver o que tem na pasta
                        crate::kinfo!("(FAT) olhando entrada:", entry.name.as_str());
                    }
                }
            }
        }
        None
    }

    #[inline]
    fn names_equal(fat_name: &str, user_name: &str) -> bool {
        let fat_name = fat_name.trim();
        let user_name = user_name.trim();

        if fat_name.eq_ignore_ascii_case(user_name) {
            return true;
        }

        let mut u_parts = user_name.splitn(2, '.');
        let u_name = u_parts.next().unwrap_or("");
        let u_ext = u_parts.next().unwrap_or("");

        let mut f_parts = fat_name.splitn(2, '.');
        let f_name = f_parts.next().unwrap_or("");
        let f_ext = f_parts.next().unwrap_or("");

        // 1. Validar Nome Base (com suporte a numeric tails ~1)
        let name_match = if f_name.contains('~') {
            let tail_idx = f_name.find('~').unwrap();
            let prefix = &f_name[..tail_idx];
            // O prefixo do disco deve bater com o início do nome buscado
            u_name.len() >= tail_idx && u_name[..tail_idx].eq_ignore_ascii_case(prefix)
        } else {
            f_name.eq_ignore_ascii_case(u_name)
        };

        // 2. Validar Extensão (truncada em 3)
        let ext_match = if u_ext.len() > 3 {
            f_ext.eq_ignore_ascii_case(&u_ext[..3])
        } else {
            f_ext.eq_ignore_ascii_case(u_ext)
        };

        if name_match && ext_match {
            crate::ktrace!("(FAT) Match 8.3 realizado com sucesso.");
            return true;
        }

        false
    }

    pub fn list_directory(&self, path: &str) -> Option<Vec<PublicDirEntry>> {
        let path = path.trim_start_matches('/');
        let mut entries = Vec::new();
        let root_cluster = if self.fat_type == FatType::Fat32 {
            self.bpb.root_cluster
        } else {
            0
        };

        let target_cluster = if path.is_empty() {
            root_cluster
        } else {
            let mut current = root_cluster;
            for component in path.split('/').filter(|s| !s.is_empty()) {
                if let Some(entry) = self.find_entry(current, component) {
                    if !entry.is_directory() {
                        return None;
                    }
                    current = entry.first_cluster();
                } else {
                    return None;
                }
            }
            current
        };

        if target_cluster == 0 && self.fat_type != FatType::Fat32 {
            self.list_root_dir(&mut entries);
        } else {
            self.list_cluster_dir(target_cluster, &mut entries);
        }
        Some(entries)
    }

    fn list_root_dir(&self, entries: &mut Vec<PublicDirEntry>) {
        let root_dir_sectors = ((self.bpb.root_entry_count as u32 * 32) + 511) / 512;
        let first_root_sector = self.partition_offset
            + self.bpb.reserved_sectors as u64
            + (self.bpb.num_fats as u64 * self.bpb.sectors_per_fat() as u64);
        let mut sector_buf = [0u8; 512];

        for i in 0..root_dir_sectors as u64 {
            if self
                .read_sector(first_root_sector + i, &mut sector_buf)
                .is_err()
            {
                continue;
            }
            for j in 0..16 {
                if let Some(entry) = DirEntry::parse(&sector_buf[j * 32..(j + 1) * 32]) {
                    let is_dir = entry.is_directory();
                    let first = entry.first_cluster();
                    entries.push(PublicDirEntry {
                        name: entry.name,
                        is_directory: is_dir,
                        size: entry.size,
                        first_cluster: first,
                    });
                }
            }
        }
    }

    fn list_cluster_dir(&self, start_cluster: u32, entries: &mut Vec<PublicDirEntry>) {
        let sectors_per_cluster = self.bpb.sectors_per_cluster as u64;
        let mut sector_buf = [0u8; 512];
        let mut cluster = start_cluster;

        loop {
            let first_sector = self.bpb.cluster_to_sector(cluster) + self.partition_offset;
            for s in 0..sectors_per_cluster {
                if self.read_sector(first_sector + s, &mut sector_buf).is_err() {
                    break;
                }
                for i in 0..16 {
                    if let Some(entry) = DirEntry::parse(&sector_buf[i * 32..(i + 1) * 32]) {
                        let is_dir = entry.is_directory();
                        let first = entry.first_cluster();
                        entries.push(PublicDirEntry {
                            name: entry.name,
                            is_directory: is_dir,
                            size: entry.size,
                            first_cluster: first,
                        });
                    }
                }
            }
            match self.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }
    }

    pub fn cluster_size(&self) -> usize {
        self.bpb.cluster_size()
    }
}
