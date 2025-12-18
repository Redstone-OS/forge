//! Testes para TmpFS (Temporary Filesystem)

#![cfg(test)]

use crate::fs::tmpfs::{TmpFS, TmpNode};

#[test]
fn test_tmpnode_file_creation() {
    let node = TmpNode::new_file("test.txt", 1024);
    assert_eq!(node.name, "test.txt");
    assert_eq!(node.is_dir, false);
    assert_eq!(node.size, 1024);
}

#[test]
fn test_tmpnode_dir_creation() {
    let node = TmpNode::new_dir("testdir");
    assert_eq!(node.name, "testdir");
    assert_eq!(node.is_dir, true);
    assert_eq!(node.size, 0);
}

#[test]
fn test_tmpfs_creation() {
    let tmpfs = TmpFS::new(1024 * 1024); // 1 MB
    assert_eq!(tmpfs.max_size, 1024 * 1024);
    assert_eq!(tmpfs.used_size, 0);
}

#[test]
fn test_tmpfs_default() {
    let tmpfs = TmpFS::default();
    assert_eq!(tmpfs.max_size, 64 * 1024 * 1024); // 64 MB
    assert_eq!(tmpfs.used_size, 0);
}

#[test]
fn test_tmpfs_available_space() {
    let tmpfs = TmpFS::new(1024);
    assert_eq!(tmpfs.available_space(), 1024);
}

// TODO: Adicionar testes quando implementar:
// - test_create_file
// - test_create_dir
// - test_remove_file
// - test_read_file
// - test_write_file
// - test_out_of_space
