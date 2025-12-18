//! Testes para DevFS (Device Filesystem)

#![cfg(test)]

use crate::fs::devfs::{DevFS, DeviceNode, DeviceNumber, DeviceType};
use crate::fs::devfs::{DEV_CONSOLE, DEV_NULL, DEV_RANDOM, DEV_ZERO};

#[test]
fn test_device_number_creation() {
    let dev = DeviceNumber::new(1, 3);
    assert_eq!(dev.major, 1);
    assert_eq!(dev.minor, 3);
}

#[test]
fn test_device_number_to_u64() {
    let dev = DeviceNumber::new(1, 3);
    // Linux format: major << 20 | minor
    let expected = (1u64 << 20) | 3;
    assert_eq!(dev.as_u64(), expected);
}

#[test]
fn test_device_node_creation() {
    let node = DeviceNode::new("null", DeviceType::Character, 1, 3);
    assert_eq!(node.name, "null");
    assert_eq!(node.device_type, DeviceType::Character);
    assert_eq!(node.dev.major, 1);
    assert_eq!(node.dev.minor, 3);
}

#[test]
fn test_standard_devices() {
    // Verifica que os dispositivos padrão têm os números corretos
    assert_eq!(DEV_NULL.major, 1);
    assert_eq!(DEV_NULL.minor, 3);

    assert_eq!(DEV_ZERO.major, 1);
    assert_eq!(DEV_ZERO.minor, 5);

    assert_eq!(DEV_RANDOM.major, 1);
    assert_eq!(DEV_RANDOM.minor, 8);

    assert_eq!(DEV_CONSOLE.major, 5);
    assert_eq!(DEV_CONSOLE.minor, 1);
}

#[test]
fn test_devfs_creation() {
    let devfs = DevFS::new();
    // DevFS deve ser criado sem erros
    let _ = devfs;
}

#[test]
fn test_device_type_equality() {
    assert_eq!(DeviceType::Character, DeviceType::Character);
    assert_eq!(DeviceType::Block, DeviceType::Block);
    assert_ne!(DeviceType::Character, DeviceType::Block);
}

// TODO: Adicionar testes quando implementar:
// - test_register_device
// - test_unregister_device
// - test_lookup_device
// - test_read_from_null
// - test_read_from_zero
// - test_write_to_null
