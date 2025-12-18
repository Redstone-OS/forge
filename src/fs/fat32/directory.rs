//! Directory - Directory entry parsing and iteration

use super::types::*;

/// Directory entry (32 bytes, packed)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; SHORT_NAME_LEN],
    pub attr: u8,
    pub reserved: u8,
    pub create_time_tenth: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub access_date: u16,
    pub first_cluster_hi: u16,
    pub modify_time: u16,
    pub modify_date: u16,
    pub first_cluster_lo: u16,
    pub size: u32,
}

impl DirEntry {
    /// Parse from 32-byte buffer
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < DIR_ENTRY_SIZE {
            return Err("Directory entry too small");
        }

        Ok(Self {
            name: [
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
                data[9], data[10],
            ],
            attr: data[11],
            reserved: data[12],
            create_time_tenth: data[13],
            create_time: u16::from_le_bytes([data[14], data[15]]),
            create_date: u16::from_le_bytes([data[16], data[17]]),
            access_date: u16::from_le_bytes([data[18], data[19]]),
            first_cluster_hi: u16::from_le_bytes([data[20], data[21]]),
            modify_time: u16::from_le_bytes([data[22], data[23]]),
            modify_date: u16::from_le_bytes([data[24], data[25]]),
            first_cluster_lo: u16::from_le_bytes([data[26], data[27]]),
            size: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
        })
    }

    /// Get first cluster
    pub const fn first_cluster(&self) -> Cluster {
        ((self.first_cluster_hi as u32) << 16) | (self.first_cluster_lo as u32)
    }

    /// Check if entry is deleted
    pub const fn is_deleted(&self) -> bool {
        self.name[0] == DIR_ENTRY_DELETED
    }

    /// Check if entry is end of directory
    pub const fn is_end(&self) -> bool {
        self.name[0] == 0
    }

    /// Check if entry is directory
    pub const fn is_directory(&self) -> bool {
        FileAttributes::new(self.attr).is_directory()
    }

    /// Check if entry is volume label
    pub const fn is_volume(&self) -> bool {
        FileAttributes::new(self.attr).is_volume()
    }

    /// Check if entry is LFN
    pub const fn is_lfn(&self) -> bool {
        FileAttributes::new(self.attr).is_lfn()
    }

    /// Get short name as string (8.3 format)
    pub fn short_name(&self) -> Result<[u8; 13], &'static str> {
        let mut result = [0u8; 13];
        let mut pos = 0;

        // Copy name part (8 bytes)
        for i in 0..8 {
            if self.name[i] != SHORT_NAME_PADDING {
                let c = if i == 0 && self.name[i] == DIR_ENTRY_E5_ENCODING {
                    0xE5
                } else {
                    self.name[i]
                };
                result[pos] = c;
                pos += 1;
            } else {
                break;
            }
        }

        // Copy extension part (3 bytes)
        let mut ext_len = 0;
        for i in 8..11 {
            if self.name[i] != SHORT_NAME_PADDING {
                ext_len = i - 8 + 1;
            } else {
                break;
            }
        }

        // Add dot and extension if present
        if ext_len > 0 {
            result[pos] = b'.';
            pos += 1;
            for i in 0..ext_len {
                result[pos] = self.name[8 + i];
                pos += 1;
            }
        }

        Ok(result)
    }

    /// Compare short name (case-insensitive)
    pub fn name_matches(&self, name: &str) -> bool {
        let Ok(short_name) = self.short_name() else {
            return false;
        };

        // Find null terminator
        let len = short_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(short_name.len());
        let short_name_str = &short_name[..len];

        // Case-insensitive comparison
        if short_name_str.len() != name.len() {
            return false;
        }

        short_name_str
            .iter()
            .zip(name.bytes())
            .all(|(a, b)| a.to_ascii_uppercase() == b.to_ascii_uppercase())
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar Long Filename (LFN)
// - Parse LFN entries
// - Reconstruct long names
// - Checksum validation

// TODO(prioridade=baixa, versão=v2.0): Implementar timestamps
// - Decode FAT date/time format
// - Convert to Unix timestamp
