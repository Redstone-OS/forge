//! Testes para SysFS (System Filesystem)

#![cfg(test)]

use crate::fs::sysfs::{SysEntry, SysEntryType, SysFS};

#[test]
fn test_sys_entry_creation() {
    let entry = SysEntry::new("cpu0", SysEntryType::Device);
    assert_eq!(entry.name, "cpu0");
    assert_eq!(entry.entry_type, SysEntryType::Device);
}

#[test]
fn test_sys_entry_types() {
    assert_eq!(SysEntryType::Device, SysEntryType::Device);
    assert_eq!(SysEntryType::Bus, SysEntryType::Bus);
    assert_ne!(SysEntryType::Device, SysEntryType::Bus);
}

#[test]
fn test_sysfs_creation() {
    let sysfs = SysFS::new();
    let _ = sysfs;
}

// TODO: Adicionar testes quando implementar:
// - test_register_device
// - test_unregister_device
// - test_read_device_attribute
// - test_write_device_attribute
// - test_list_devices
