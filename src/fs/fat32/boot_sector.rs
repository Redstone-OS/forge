//! Boot Sector - BIOS Parameter Block parsing and validation

use super::types::*;

/// BIOS Parameter Block (FAT32)
#[derive(Debug, Clone)]
pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries: u16,     // Must be 0 for FAT32
    pub total_sectors_16: u16, // Must be 0 for FAT32
    pub media: u8,
    pub sectors_per_fat_16: u16, // Must be 0 for FAT32
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,

    // FAT32 specific fields
    pub sectors_per_fat_32: u32,
    pub extended_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info_sector: u16,
    pub backup_boot_sector: u16,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
}

impl BiosParameterBlock {
    /// Parse BPB from boot sector (512 bytes)
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 512 {
            return Err("Boot sector too small");
        }

        // Check boot signature
        if data[510] != 0x55 || data[511] != 0xAA {
            return Err("Invalid boot signature");
        }

        // Parse BPB fields (little-endian)
        let bpb = Self {
            bytes_per_sector: u16::from_le_bytes([data[11], data[12]]),
            sectors_per_cluster: data[13],
            reserved_sectors: u16::from_le_bytes([data[14], data[15]]),
            num_fats: data[16],
            root_entries: u16::from_le_bytes([data[17], data[18]]),
            total_sectors_16: u16::from_le_bytes([data[19], data[20]]),
            media: data[21],
            sectors_per_fat_16: u16::from_le_bytes([data[22], data[23]]),
            sectors_per_track: u16::from_le_bytes([data[24], data[25]]),
            num_heads: u16::from_le_bytes([data[26], data[27]]),
            hidden_sectors: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            total_sectors_32: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),

            // FAT32 specific (offset 36)
            sectors_per_fat_32: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
            extended_flags: u16::from_le_bytes([data[40], data[41]]),
            fs_version: u16::from_le_bytes([data[42], data[43]]),
            root_cluster: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
            fs_info_sector: u16::from_le_bytes([data[48], data[49]]),
            backup_boot_sector: u16::from_le_bytes([data[50], data[51]]),
            volume_id: u32::from_le_bytes([data[67], data[68], data[69], data[70]]),
            volume_label: [
                data[71], data[72], data[73], data[74], data[75], data[76], data[77], data[78],
                data[79], data[80], data[81],
            ],
        };

        bpb.validate()?;
        Ok(bpb)
    }

    /// Validate BPB fields
    pub fn validate(&self) -> Result<(), &'static str> {
        // Must be FAT32
        if self.sectors_per_fat_16 != 0 {
            return Err("Not FAT32 (sectors_per_fat_16 != 0)");
        }

        // Bytes per sector must be power of 2 and in range [512, 4096]
        if !self.bytes_per_sector.is_power_of_two() {
            return Err("bytes_per_sector not power of 2");
        }
        if self.bytes_per_sector < 512 || self.bytes_per_sector > 4096 {
            return Err("bytes_per_sector out of range");
        }

        // Sectors per cluster must be power of 2
        if !self.sectors_per_cluster.is_power_of_two() {
            return Err("sectors_per_cluster not power of 2");
        }

        // Cluster size must be <= 32KB
        let cluster_size = u32::from(self.bytes_per_sector) * u32::from(self.sectors_per_cluster);
        if cluster_size > 32 * 1024 {
            return Err("cluster size > 32KB");
        }

        // Must have at least 1 reserved sector
        if self.reserved_sectors < 1 {
            return Err("reserved_sectors < 1");
        }

        // Must have at least 1 FAT
        if self.num_fats == 0 {
            return Err("num_fats == 0");
        }

        // FAT32 must have root_entries == 0
        if self.root_entries != 0 {
            return Err("FAT32 must have root_entries == 0");
        }

        // FAT32 must have total_sectors_16 == 0
        if self.total_sectors_16 != 0 {
            return Err("FAT32 must have total_sectors_16 == 0");
        }

        // Must have sectors_per_fat_32 > 0
        if self.sectors_per_fat_32 == 0 {
            return Err("sectors_per_fat_32 == 0");
        }

        // Total sectors must be valid
        if self.total_sectors_32 == 0 {
            return Err("total_sectors_32 == 0");
        }

        Ok(())
    }

    /// Get first FAT sector
    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sectors as u32
    }

    /// Get first data sector
    pub fn data_start_sector(&self) -> u32 {
        let fat_sectors = (self.num_fats as u32) * self.sectors_per_fat_32;
        (self.reserved_sectors as u32) + fat_sectors
    }

    /// Get total clusters
    pub fn total_clusters(&self) -> u32 {
        let data_sectors = self.total_sectors_32 - self.data_start_sector();
        data_sectors / (self.sectors_per_cluster as u32)
    }

    /// Get cluster size in bytes
    pub fn cluster_size(&self) -> u32 {
        (self.bytes_per_sector as u32) * (self.sectors_per_cluster as u32)
    }

    /// Convert cluster to first sector
    pub fn cluster_to_sector(&self, cluster: Cluster) -> Sector {
        if cluster < RESERVED_FAT_ENTRIES {
            return 0; // Invalid cluster
        }
        let cluster_offset = cluster - RESERVED_FAT_ENTRIES;
        self.data_start_sector() + (cluster_offset * (self.sectors_per_cluster as u32))
    }
}
