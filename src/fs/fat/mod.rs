//! # Driver de Sistema de Arquivos FAT
//!
//! Suporta FAT16 e FAT32 para leitura de arquivos do disco.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  Boot Sector (BPB) │  FAT Table  │  Root Dir │ Data  │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Estrutura do Módulo
//!
//! - `bpb.rs` - Parser do BIOS Parameter Block (boot sector)
//! - `dir.rs` - Parsing de entradas de diretório
//! - `file.rs` - Operações de leitura de arquivos
//! - `file.rs` - Operações de leitura de arquivos
//! - `fs.rs` - Struct principal FatFs e montagem

pub mod bpb;
pub mod dir;
pub mod file;
pub mod fs;

// Re-exports públicos
pub use fs::{FatFs, FatType};

use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// INSTÂNCIA GLOBAL
// =============================================================================

/// Instância global do FAT montado
static MOUNTED_FAT: Spinlock<Option<FatFs>> = Spinlock::new(None);

// =============================================================================
// API PÚBLICA
// =============================================================================

/// Inicializa o módulo FAT e tenta montar o primeiro disco
pub fn init() {
    crate::kinfo!("(FAT) Inicializando módulo...");

    // Tentar montar o primeiro dispositivo de bloco
    if let Some(device) = crate::drivers::block::first_device() {
        match FatFs::mount(device) {
            Ok(fat) => {
                crate::kinfo!("(FAT) Filesystem montado com sucesso!");
                *MOUNTED_FAT.lock() = Some(fat);
            }
            Err(e) => {
                crate::kwarn!("(FAT) Falha ao montar:", e as u64);
            }
        }
    } else {
        crate::kwarn!("(FAT) Nenhum dispositivo de bloco disponível");
    }
}

/// Lê um arquivo do FAT montado
pub fn read_file(path: &str) -> Option<Vec<u8>> {
    let guard = MOUNTED_FAT.lock();
    if let Some(fat) = guard.as_ref() {
        fat.read_file(path)
    } else {
        None
    }
}

/// Lista entradas de um diretório do FAT montado
pub fn list_directory(path: &str) -> Option<Vec<PublicDirEntry>> {
    let guard = MOUNTED_FAT.lock();
    if let Some(fat) = guard.as_ref() {
        fat.list_directory(path)
    } else {
        None
    }
}

// =============================================================================
// TIPOS PÚBLICOS
// =============================================================================

/// Entrada de diretório pública (para syscalls)
#[derive(Debug, Clone)]
pub struct PublicDirEntry {
    pub name: String,
    pub is_directory: bool,
    pub size: u32,
    pub first_cluster: u32,
}
