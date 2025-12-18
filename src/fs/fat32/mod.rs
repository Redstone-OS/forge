//! FAT32 - File Allocation Table 32-bit Filesystem
//!
//! Implementação read-only do sistema de arquivos FAT32.
//!
//! # Características
//! - ✅ FAT32 read-only
//! - ✅ Short names (8.3)
//! - ✅ Navegação de diretórios
//! - ✅ Leitura de arquivos
//! - ⏳ Long filenames (LFN) - TODO
//! - ⏳ Escrita - TODO
//!
//! # Uso
//!
//! ```rust,ignore
//! // 1. Ler boot sector do dispositivo
//! let boot_sector = read_sector(device, 0)?;
//!
//! // 2. Montar filesystem
//! let fat32 = Fat32::mount(&boot_sector, device)?;
//!
//! // 3. Abrir arquivo
//! let file = fat32.open("/boot/kernel")?;
//!
//! // 4. Ler dados
//! let mut buffer = [0u8; 4096];
//! let bytes_read = file.read(&mut buffer)?;
//! ```

#![allow(dead_code)]

pub mod boot_sector;
pub mod cluster;
pub mod directory;
pub mod types;

pub use boot_sector::BiosParameterBlock;
pub use directory::DirEntry;
pub use types::*;

/// FAT32 Filesystem
pub struct Fat32 {
    /// BIOS Parameter Block
    bpb: BiosParameterBlock,
    /// FAT table cache (first 4KB)
    fat_cache: [u8; 4096],
    // TODO: Add block device reference when available
}

impl Fat32 {
    /// Mount FAT32 filesystem
    ///
    /// # Arguments
    /// * `boot_sector` - 512-byte boot sector data
    ///
    /// # Returns
    /// Mounted FAT32 filesystem
    pub fn mount(boot_sector: &[u8]) -> Result<Self, &'static str> {
        let bpb = BiosParameterBlock::parse(boot_sector)?;

        Ok(Self {
            bpb,
            fat_cache: [0; 4096],
        })
    }

    /// Get BIOS Parameter Block
    pub fn bpb(&self) -> &BiosParameterBlock {
        &self.bpb
    }

    /// Get root directory cluster
    pub fn root_cluster(&self) -> Cluster {
        self.bpb.root_cluster
    }

    /// Get FAT cache
    pub fn fat_cache(&self) -> &[u8] {
        &self.fat_cache
    }

    /// Set FAT cache data
    ///
    /// # Arguments
    /// * `data` - FAT data to cache (up to 4KB)
    pub fn set_fat_cache(&mut self, data: &[u8]) {
        let len = data.len().min(self.fat_cache.len());
        self.fat_cache[..len].copy_from_slice(&data[..len]);
    }

    /// Read FAT entry for cluster
    pub fn read_fat_entry(&self, cluster: Cluster) -> Result<FatValue, &'static str> {
        cluster::read_fat_entry(&self.fat_cache, cluster)
    }

    /// Get next cluster in chain
    pub fn next_cluster(&self, cluster: Cluster) -> Result<Option<Cluster>, &'static str> {
        cluster::get_next_cluster(&self.fat_cache, cluster)
    }

    /// Create cluster chain iterator
    pub fn cluster_chain(&self, start_cluster: Cluster) -> cluster::ClusterChain {
        cluster::ClusterChain::new(&self.fat_cache, start_cluster)
    }

    /// Convert cluster to sector number
    pub fn cluster_to_sector(&self, cluster: Cluster) -> Sector {
        self.bpb.cluster_to_sector(cluster)
    }

    /// Get cluster size in bytes
    pub fn cluster_size(&self) -> u32 {
        self.bpb.cluster_size()
    }
}

impl Default for Fat32 {
    fn default() -> Self {
        Self {
            bpb: BiosParameterBlock {
                bytes_per_sector: 512,
                sectors_per_cluster: 1,
                reserved_sectors: 1,
                num_fats: 2,
                root_entries: 0,
                total_sectors_16: 0,
                media: 0xF8,
                sectors_per_fat_16: 0,
                sectors_per_track: 0,
                num_heads: 0,
                hidden_sectors: 0,
                total_sectors_32: 0,
                sectors_per_fat_32: 0,
                extended_flags: 0,
                fs_version: 0,
                root_cluster: 2,
                fs_info_sector: 1,
                backup_boot_sector: 6,
                volume_id: 0,
                volume_label: [0; 11],
            },
            fat_cache: [0; 4096],
        }
    }
}

// TODO(prioridade=alta, versão=v1.0): Integrar com block device
// - Adicionar referência ao block device
// - Implementar leitura de setores
// - Implementar cache de setores

// TODO(prioridade=média, versão=v1.0): Implementar API de alto nível
// - open(path) -> File
// - read_dir(path) -> Iterator<DirEntry>
// - read_file(path, buf) -> Result<usize>

// TODO(prioridade=média, versão=v1.0): Implementar File handle
// - read() - Ler dados do arquivo
// - seek() - Mover posição de leitura
// - size() - Obter tamanho do arquivo

// TODO(prioridade=baixa, versão=v2.0): Implementar escrita
// - write() - Escrever dados
// - create() - Criar arquivo
// - delete() - Deletar arquivo
// - mkdir() - Criar diretório
