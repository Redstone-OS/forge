//! Cluster - FAT navigation and cluster chain iteration

use super::boot_sector::BiosParameterBlock;
use super::types::*;

/// Read FAT32 entry from FAT table
pub fn read_fat_entry(fat_data: &[u8], cluster: Cluster) -> Result<FatValue, &'static str> {
    let offset = (cluster * 4) as usize;

    if offset + 4 > fat_data.len() {
        return Err("FAT entry out of bounds");
    }

    let val = u32::from_le_bytes([
        fat_data[offset],
        fat_data[offset + 1],
        fat_data[offset + 2],
        fat_data[offset + 3],
    ]);

    Ok(FatValue::from_u32(val))
}

/// Get next cluster in chain
pub fn get_next_cluster(
    fat_data: &[u8],
    cluster: Cluster,
) -> Result<Option<Cluster>, &'static str> {
    match read_fat_entry(fat_data, cluster)? {
        FatValue::Data(next) => Ok(Some(next)),
        FatValue::EndOfChain => Ok(None),
        FatValue::Free => Err("Cluster is free"),
        FatValue::Bad => Err("Bad cluster"),
    }
}

/// Cluster chain iterator
pub struct ClusterChain<'a> {
    fat_data: &'a [u8],
    current: Option<Cluster>,
}

impl<'a> ClusterChain<'a> {
    /// Create new cluster chain starting at given cluster
    pub const fn new(fat_data: &'a [u8], start_cluster: Cluster) -> Self {
        Self {
            fat_data,
            current: Some(start_cluster),
        }
    }

    /// Get current cluster
    pub const fn current(&self) -> Option<Cluster> {
        self.current
    }
}

impl Iterator for ClusterChain<'_> {
    type Item = Result<Cluster, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        // Return current cluster
        let result = current;

        // Move to next cluster
        self.current = match get_next_cluster(self.fat_data, current) {
            Ok(next) => next,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(result))
    }
}

/// Calculate cluster offset for a given file position
pub fn position_to_cluster_offset(bpb: &BiosParameterBlock, position: u32) -> (u32, u32) {
    let cluster_size = bpb.cluster_size();
    let cluster_index = position / cluster_size;
    let offset_in_cluster = position % cluster_size;
    (cluster_index, offset_in_cluster)
}

// TODO(prioridade=baixa, versão=v2.0): Implementar cache de FAT
// - LRU cache para entradas da FAT
// - Reduzir leituras de disco
