//! Testes para FAT32 Filesystem

#![cfg(test)]

use crate::fs::fat32::{Fat32, Fat32BootSector, Fat32DirEntry};
use crate::fs::fat32::{ATTR_READ_ONLY, ATTR_DIRECTORY, ATTR_ARCHIVE};

#[test]
fn test_fat32_creation() {
    let fat32 = Fat32::new();
    let _ = fat32;
}

#[test]
fn test_fat32_default() {
    let fat32 = Fat32::default();
    let _ = fat32;
}

#[test]
fn test_fat32_attributes() {
    assert_eq!(ATTR_READ_ONLY, 0x01);
    assert_eq!(ATTR_DIRECTORY, 0x10);
    assert_eq!(ATTR_ARCHIVE, 0x20);
}

#[test]
fn test_boot_sector_size() {
    // Boot sector deve ter exatamente 512 bytes
    assert_eq!(core::mem::size_of::<Fat32BootSector>(), 90);
    // Nota: O boot sector completo tem 512 bytes, mas nossa struct
    // tem apenas os campos principais (90 bytes)
}

#[test]
fn test_dir_entry_size() {
    // Entrada de diretório deve ter exatamente 32 bytes
    assert_eq!(core::mem::size_of::<Fat32DirEntry>(), 32);
}

// TODO: Adicionar testes quando implementar:
// - test_mount_volume
// - test_read_boot_sector
// - test_read_fat
// - test_read_directory
// - test_read_file
// - test_parse_long_filename
