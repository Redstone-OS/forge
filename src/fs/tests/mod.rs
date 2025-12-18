//! Testes para o módulo Filesystem
//!
//! Este módulo contém testes unitários e de integração para todos
//! os componentes do sistema de arquivos.
//!
//! # Como Executar os Testes
//!
//! ## Testes Unitários (no host)
//! ```bash
//! # Executar todos os testes de filesystem
//! cargo test --package forge --lib fs::tests
//!
//! # Executar testes de um módulo específico
//! cargo test --package forge --lib fs::tests::devfs
//! cargo test --package forge --lib fs::tests::vfs
//!
//! # Executar um teste específico
//! cargo test --package forge --lib fs::tests::devfs::test_device_number
//! ```
//!
//! ## Testes de Integração (no kernel)
//! ```bash
//! # Executar testes no QEMU
//! cargo test --package forge --target x86_64-unknown-none
//! ```
//!
//! # Estrutura dos Testes
//!
//! - `devfs.rs` - Testes do DevFS
//! - `procfs.rs` - Testes do ProcFS
//! - `sysfs.rs` - Testes do SysFS
//! - `tmpfs.rs` - Testes do TmpFS
//! - `fat32.rs` - Testes do FAT32
//! - `vfs.rs` - Testes do VFS
//! - `integration.rs` - Testes de integração entre módulos
//!
//! # Convenções
//!
//! - Prefixo `test_` para testes unitários
//! - Prefixo `integration_` para testes de integração
//! - Prefixo `bench_` para benchmarks (quando disponível)
//! - Use `#[should_panic]` para testes que devem falhar
//! - Use `#[ignore]` para testes que requerem hardware específico

#![cfg(test)]

// Módulos de teste
pub mod devfs;
pub mod fat32;
pub mod integration;
pub mod procfs;
pub mod sysfs;
pub mod tmpfs;
pub mod vfs;

// Re-exports úteis para testes
pub use crate::fs::{devfs::DevFS, fat32::Fat32, procfs::ProcFS, sysfs::SysFS, tmpfs::TmpFS};

/// Helper: Cria um DevFS para testes
pub fn create_test_devfs() -> DevFS {
    DevFS::new()
}

/// Helper: Cria um ProcFS para testes
pub fn create_test_procfs() -> ProcFS {
    ProcFS::new()
}

/// Helper: Cria um SysFS para testes
pub fn create_test_sysfs() -> SysFS {
    SysFS::new()
}

/// Helper: Cria um TmpFS para testes (1MB)
pub fn create_test_tmpfs() -> TmpFS {
    TmpFS::new(1024 * 1024) // 1 MB
}

/// Helper: Cria um Fat32 para testes
pub fn create_test_fat32() -> Fat32 {
    Fat32::new()
}

// Testes básicos do módulo
#[test]
fn test_module_exists() {
    // Verifica que o módulo de testes existe e compila
    assert!(true);
}
