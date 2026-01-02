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

use crate::drivers::block::BlockDevice;
use crate::fs::vfs::inode::FsError;
use crate::sync::Spinlock;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Instância global do FAT montado
static MOUNTED_FAT: Spinlock<Option<FatFs>> = Spinlock::new(None);

/// Instância do filesystem FAT
pub struct FatFs {
    /// Dispositivo de bloco subjacente
    device: Arc<dyn BlockDevice>,
    /// Informações do BPB
    bpb: bpb::Bpb,
    /// Tipo de FAT
    fat_type: FatType,
    /// Offset da partição (LBA do início do FAT)
    partition_offset: u64,
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

        // Ler setor 0 (pode ser MBR ou boot sector direto)
        let mut sector0 = [0u8; 512];
        device
            .read_block(0, &mut sector0)
            .map_err(|_| FsError::IoError)?;

        // Debug: mostrar primeiros bytes
        crate::kdebug!(
            "(FAT) Sector[0..4]:",
            u32::from_le_bytes([sector0[0], sector0[1], sector0[2], sector0[3]]) as u64
        );
        crate::kdebug!(
            "(FAT) Sector[510,511]:",
            ((sector0[510] as u64) << 8) | (sector0[511] as u64)
        );

        // Determinar se é MBR ou FAT boot sector
        // FAT boot sector começa com jump instruction: 0xEB ou 0xE9
        // MBR/partitioned disk tem bytes 0-2 como zeros ou código de boot
        let partition_start = if sector0[0] == 0xEB || sector0[0] == 0xE9 {
            // Parece FAT boot sector direto
            crate::kinfo!("(FAT) Boot sector direto detectado");
            0u64
        } else if sector0[510] == 0x55 && sector0[511] == 0xAA {
            // Provavelmente MBR com tabela de partições
            crate::kinfo!("(FAT) MBR detectado, buscando partição...");

            // Partições começam em offset 0x1BE (446) no MBR
            // Cada entrada tem 16 bytes
            let part_entry = &sector0[0x1BE..0x1BE + 16];

            // Offset 0x08-0x0B: LBA do início da partição
            let lba_start =
                u32::from_le_bytes([part_entry[8], part_entry[9], part_entry[10], part_entry[11]])
                    as u64;

            crate::kinfo!("(FAT) Partição 1 começa em LBA:", lba_start);

            if lba_start == 0 {
                crate::kwarn!("(FAT) Partição inválida (LBA=0)");
                return Err(FsError::InvalidFormat);
            }

            lba_start
        } else {
            crate::kwarn!("(FAT) Formato desconhecido");
            return Err(FsError::InvalidFormat);
        };

        // Ler o verdadeiro FAT boot sector
        let mut boot_sector = [0u8; 512];
        device
            .read_block(partition_start, &mut boot_sector)
            .map_err(|_| FsError::IoError)?;

        crate::kdebug!(
            "(FAT) FAT Boot sector[0..4]:",
            u32::from_le_bytes([
                boot_sector[0],
                boot_sector[1],
                boot_sector[2],
                boot_sector[3]
            ]) as u64
        );

        // Parsear BPB
        let bpb = match bpb::Bpb::parse(&boot_sector) {
            Some(b) => b,
            None => {
                crate::kwarn!("(FAT) BPB inválido ou disco não formatado");
                return Err(FsError::InvalidFormat);
            }
        };

        // Validar valores críticos antes de usar
        if bpb.bytes_per_sector == 0 {
            crate::kwarn!("(FAT) bytes_per_sector = 0, inválido!");
            return Err(FsError::InvalidFormat);
        }
        if bpb.sectors_per_cluster == 0 {
            crate::kwarn!("(FAT) sectors_per_cluster = 0, inválido!");
            return Err(FsError::InvalidFormat);
        }

        crate::kinfo!("(FAT) Bytes por setor:", bpb.bytes_per_sector as u64);
        crate::kinfo!("(FAT) Setores por cluster:", bpb.sectors_per_cluster as u64);
        crate::kinfo!("(FAT) Reserved sectors:", bpb.reserved_sectors as u64);
        crate::kinfo!("(FAT) Num FATs:", bpb.num_fats as u64);

        // Determinar tipo de FAT
        let fat_type = bpb.fat_type();

        crate::kinfo!("(FAT) Tipo:", fat_type as u64);

        Ok(Self {
            device,
            bpb,
            fat_type,
            partition_offset: partition_start,
        })
    }

    /// Lê um cluster inteiro para um buffer
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

    /// Obtém o próximo cluster na cadeia da tabela FAT
    pub fn next_cluster(&self, cluster: u32) -> Option<u32> {
        let fat_offset = match self.fat_type {
            FatType::Fat12 => (cluster + (cluster / 2)) as usize,
            FatType::Fat16 => (cluster * 2) as usize,
            FatType::Fat32 => (cluster * 4) as usize,
        };

        let fat_sector =
            self.partition_offset + self.bpb.reserved_sectors as u64 + (fat_offset / 512) as u64;
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

    /// Lê um arquivo pelo caminho (relativo à raiz do FAT)
    /// Retorna o conteúdo do arquivo como Vec<u8>
    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        crate::ktrace!("(FAT) read_file:", path.as_ptr() as u64);

        // Normalizar path - remover leading slashes
        let path = path.trim_start_matches('/');

        // Separar path em componentes
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if components.is_empty() {
            return None;
        }

        // Começar da raiz
        let root_cluster = if self.fat_type == FatType::Fat32 {
            self.bpb.root_cluster
        } else {
            0 // FAT12/16 usam área de root fixa
        };

        // Navegar pelos diretórios
        let mut current_cluster = root_cluster;

        for (i, component) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;

            // Buscar entrada no diretório atual
            if let Some(entry) = self.find_entry(current_cluster, component) {
                if is_last {
                    // É o arquivo final - ler conteúdo
                    if entry.is_directory {
                        return None; // Esperávamos arquivo, não diretório
                    }
                    return self.read_file_data(entry.first_cluster, entry.size);
                } else {
                    // É um diretório intermediário
                    if !entry.is_directory {
                        return None; // Esperávamos diretório
                    }
                    current_cluster = entry.first_cluster;
                }
            } else {
                crate::ktrace!("(FAT) Componente não encontrado");
                return None;
            }
        }

        None
    }

    /// Busca uma entrada no diretório
    fn find_entry(&self, dir_cluster: u32, name: &str) -> Option<DirEntry> {
        let name_upper: String = name.chars().map(|c| c.to_ascii_uppercase()).collect();

        // Se for FAT12/16 e cluster 0, ler root directory area
        if dir_cluster == 0 && self.fat_type != FatType::Fat32 {
            return self.find_in_root_dir(&name_upper);
        }

        // Ler clusters do diretório
        let cluster_size = self.bpb.cluster_size();
        let mut buf = alloc::vec![0u8; cluster_size];
        let mut cluster = dir_cluster;

        loop {
            if self.read_cluster(cluster, &mut buf).is_err() {
                return None;
            }

            // Parsear entradas de diretório (32 bytes cada)
            for i in 0..(cluster_size / 32) {
                let offset = i * 32;
                if let Some(entry) = self.parse_dir_entry(&buf[offset..offset + 32]) {
                    if entry.name == name_upper {
                        return Some(entry);
                    }
                }
            }

            // Próximo cluster
            match self.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }

        None
    }

    /// Busca na área de root directory (FAT12/16)
    fn find_in_root_dir(&self, name: &str) -> Option<DirEntry> {
        let root_dir_sectors = ((self.bpb.root_entry_count as u32 * 32) + 511) / 512;
        let first_root_sector = self.partition_offset
            + self.bpb.reserved_sectors as u64
            + (self.bpb.num_fats as u64 * self.bpb.sectors_per_fat() as u64);

        let mut sector_buf = [0u8; 512];

        for i in 0..root_dir_sectors as u64 {
            if self
                .device
                .read_block(first_root_sector + i, &mut sector_buf)
                .is_err()
            {
                continue;
            }

            // 16 entradas por setor
            for j in 0..16 {
                let offset = j * 32;
                if let Some(entry) = self.parse_dir_entry(&sector_buf[offset..offset + 32]) {
                    if entry.name == name {
                        return Some(entry);
                    }
                }
            }
        }

        None
    }

    /// Parseia uma entrada de diretório de 32 bytes
    fn parse_dir_entry(&self, data: &[u8]) -> Option<DirEntry> {
        if data.len() < 32 {
            return None;
        }

        // Primeiro byte 0x00 = fim, 0xE5 = deletada
        if data[0] == 0x00 || data[0] == 0xE5 {
            return None;
        }

        // Atributos
        let attr = data[11];

        // Pular LFN entries
        if attr == 0x0F {
            return None;
        }

        // Nome 8.3
        let name_bytes = &data[0..8];
        let ext_bytes = &data[8..11];

        // Converter para string
        let name_part: String = name_bytes
            .iter()
            .take_while(|&&b| b != 0x20 && b != 0x00)
            .map(|&b| b as char)
            .collect();

        let ext_part: String = ext_bytes
            .iter()
            .take_while(|&&b| b != 0x20 && b != 0x00)
            .map(|&b| b as char)
            .collect();

        let name = if ext_part.is_empty() {
            name_part
        } else {
            alloc::format!("{}.{}", name_part, ext_part)
        };

        // Primeiro cluster
        let cluster_hi = u16::from_le_bytes([data[20], data[21]]) as u32;
        let cluster_lo = u16::from_le_bytes([data[26], data[27]]) as u32;
        let first_cluster = (cluster_hi << 16) | cluster_lo;

        // Tamanho
        let size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        Some(DirEntry {
            name,
            is_directory: (attr & 0x10) != 0,
            first_cluster,
            size,
        })
    }

    /// Lê o conteúdo de um arquivo dado seu primeiro cluster
    fn read_file_data(&self, first_cluster: u32, size: u32) -> Option<Vec<u8>> {
        let mut data = Vec::with_capacity(size as usize);
        let cluster_size = self.bpb.cluster_size();
        let mut buf = alloc::vec![0u8; cluster_size];
        let mut remaining = size as usize;
        let mut cluster = first_cluster;

        loop {
            if self.read_cluster(cluster, &mut buf).is_err() {
                return None;
            }

            let to_copy = remaining.min(cluster_size);
            data.extend_from_slice(&buf[..to_copy]);
            remaining = remaining.saturating_sub(cluster_size);

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

    /// Lista entradas de um diretório
    ///
    /// # Args
    /// - path: caminho do diretório (relativo à raiz do FAT)
    ///
    /// # Returns
    /// Lista de entradas ou None se não for um diretório válido
    pub fn list_directory(&self, path: &str) -> Option<Vec<PublicDirEntry>> {
        let path = path.trim_start_matches('/');
        let mut entries = Vec::new();

        // Se path vazio, listar raiz
        let root_cluster = if self.fat_type == FatType::Fat32 {
            self.bpb.root_cluster
        } else {
            0
        };

        let target_cluster = if path.is_empty() {
            root_cluster
        } else {
            // Navegar até o diretório alvo
            let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let mut current = root_cluster;

            for component in components {
                if let Some(entry) = self.find_entry(current, component) {
                    if !entry.is_directory {
                        return None; // Não é diretório
                    }
                    current = entry.first_cluster;
                } else {
                    return None; // Não encontrado
                }
            }
            current
        };

        // Ler entradas do diretório
        if target_cluster == 0 && self.fat_type != FatType::Fat32 {
            // Root dir area para FAT12/16
            self.list_root_dir(&mut entries);
        } else {
            self.list_cluster_dir(target_cluster, &mut entries);
        }

        Some(entries)
    }

    /// Lista entradas do root directory (FAT12/16)
    fn list_root_dir(&self, entries: &mut Vec<PublicDirEntry>) {
        let root_dir_sectors = ((self.bpb.root_entry_count as u32 * 32) + 511) / 512;
        let first_root_sector = self.partition_offset
            + self.bpb.reserved_sectors as u64
            + (self.bpb.num_fats as u64 * self.bpb.sectors_per_fat() as u64);

        let mut sector_buf = [0u8; 512];

        for i in 0..root_dir_sectors as u64 {
            if self
                .device
                .read_block(first_root_sector + i, &mut sector_buf)
                .is_err()
            {
                continue;
            }

            for j in 0..16 {
                let offset = j * 32;
                if let Some(entry) = self.parse_dir_entry(&sector_buf[offset..offset + 32]) {
                    entries.push(PublicDirEntry {
                        name: entry.name,
                        is_directory: entry.is_directory,
                        size: entry.size,
                        first_cluster: entry.first_cluster,
                    });
                }
            }
        }
    }

    /// Lista entradas de um diretório em clusters (FAT32 ou subdir)
    fn list_cluster_dir(&self, start_cluster: u32, entries: &mut Vec<PublicDirEntry>) {
        let cluster_size = self.bpb.cluster_size();
        let mut buf = alloc::vec![0u8; cluster_size];
        let mut cluster = start_cluster;

        loop {
            if self.read_cluster(cluster, &mut buf).is_err() {
                break;
            }

            for i in 0..(cluster_size / 32) {
                let offset = i * 32;
                if let Some(entry) = self.parse_dir_entry(&buf[offset..offset + 32]) {
                    entries.push(PublicDirEntry {
                        name: entry.name,
                        is_directory: entry.is_directory,
                        size: entry.size,
                        first_cluster: entry.first_cluster,
                    });
                }
            }

            match self.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }
    }
}

/// Entrada de diretório interna
struct DirEntry {
    name: String,
    is_directory: bool,
    first_cluster: u32,
    size: u32,
}

/// Entrada de diretório pública (para syscalls)
#[derive(Debug, Clone)]
pub struct PublicDirEntry {
    pub name: String,
    pub is_directory: bool,
    pub size: u32,
    pub first_cluster: u32,
}

/// Inicializa o módulo FAT e tenta montar o primeiro disco
pub fn init() {
    crate::kinfo!("(FAT) Inicializando módulo...");

    // Tentar montar o primeiro dispositivo de bloco
    if let Some(device) = crate::drivers::block::first_device() {
        match FatFs::mount(device) {
            Ok(fat) => {
                crate::kinfo!("(FAT) Filesystem montado com sucesso!");
                *MOUNTED_FAT.lock() = Some(fat);
            }
            Err(e) => {
                crate::kwarn!("(FAT) Falha ao montar:", e as u64);
            }
        }
    } else {
        crate::kwarn!("(FAT) Nenhum dispositivo de bloco disponível");
    }
}

/// Lê um arquivo do FAT montado
pub fn read_file(path: &str) -> Option<Vec<u8>> {
    let guard = MOUNTED_FAT.lock();
    if let Some(fat) = guard.as_ref() {
        fat.read_file(path)
    } else {
        None
    }
}

/// Lista entradas de um diretório do FAT montado
pub fn list_directory(path: &str) -> Option<Vec<PublicDirEntry>> {
    let guard = MOUNTED_FAT.lock();
    if let Some(fat) = guard.as_ref() {
        fat.list_directory(path)
    } else {
        None
    }
}
