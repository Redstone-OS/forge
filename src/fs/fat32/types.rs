//! FAT32 Types - Common types and constants

#![allow(dead_code)]

/// FAT type (only FAT32 supported)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatType {
    Fat32,
}

/// Cluster number
pub type Cluster = u32;

/// Sector number  
pub type Sector = u32;

/// Reserved FAT entries (0 and 1)
pub const RESERVED_FAT_ENTRIES: u32 = 2;

/// FAT entry value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatValue {
    /// Free cluster
    Free,
    /// Data cluster (points to next cluster)
    Data(Cluster),
    /// Bad cluster
    Bad,
    /// End of cluster chain
    EndOfChain,
}

impl FatValue {
    /// Parse FAT32 entry (28 bits used)
    pub const fn from_u32(val: u32) -> Self {
        let val = val & 0x0FFF_FFFF;
        match val {
            0 => Self::Free,
            0x0FFF_FFF7 => Self::Bad,
            0x0FFF_FFF8..=0x0FFF_FFFF => Self::EndOfChain,
            n => Self::Data(n),
        }
    }
}

/// File attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileAttributes {
    bits: u8,
}

impl FileAttributes {
    pub const READ_ONLY: u8 = 0x01;
    pub const HIDDEN: u8 = 0x02;
    pub const SYSTEM: u8 = 0x04;
    pub const VOLUME_ID: u8 = 0x08;
    pub const DIRECTORY: u8 = 0x10;
    pub const ARCHIVE: u8 = 0x20;
    pub const LFN: u8 = Self::READ_ONLY | Self::HIDDEN | Self::SYSTEM | Self::VOLUME_ID;

    pub const fn new(bits: u8) -> Self {
        Self { bits }
    }

    pub const fn is_directory(&self) -> bool {
        self.bits & Self::DIRECTORY != 0
    }

    pub const fn is_volume(&self) -> bool {
        self.bits & Self::VOLUME_ID != 0
    }

    pub const fn is_lfn(&self) -> bool {
        self.bits & Self::LFN == Self::LFN
    }
}

/// Directory entry size in bytes
pub const DIR_ENTRY_SIZE: usize = 32;

/// Deleted entry marker
pub const DIR_ENTRY_DELETED: u8 = 0xE5;

/// Special encoding for 0xE5 in first byte
pub const DIR_ENTRY_E5_ENCODING: u8 = 0x05;

/// Short filename size (8.3 format)
pub const SHORT_NAME_LEN: usize = 11;

/// Short filename padding
pub const SHORT_NAME_PADDING: u8 = b' ';
