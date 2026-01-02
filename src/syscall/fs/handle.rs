//! # File Handle Management
//!
//! Gerenciamento de handles de arquivo abertos por processo.

use super::types::{FileType, OpenFlags};
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// FILE HANDLE
// =============================================================================

/// Handle de arquivo aberto
#[derive(Debug)]
pub struct FileHandle {
    /// Path do arquivo (para debug e stat)
    pub path: String,
    /// Tipo de arquivo
    pub file_type: FileType,
    /// Flags de abertura
    pub flags: OpenFlags,
    /// Posição atual de leitura/escrita
    pub offset: u64,
    /// Tamanho do arquivo
    pub size: u64,
    /// Primeiro cluster (para FAT)
    pub first_cluster: u32,
    /// Índice atual para readdir (se diretório)
    pub dir_index: usize,
}

impl FileHandle {
    pub fn new(
        path: String,
        file_type: FileType,
        flags: OpenFlags,
        size: u64,
        first_cluster: u32,
    ) -> Self {
        Self {
            path,
            file_type,
            flags,
            offset: 0,
            size,
            first_cluster,
            dir_index: 0,
        }
    }

    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }

    pub fn can_read(&self) -> bool {
        self.flags.can_read()
    }

    pub fn can_write(&self) -> bool {
        self.flags.can_write()
    }
}

// =============================================================================
// HANDLE TABLE
// =============================================================================

/// Tabela de handles de arquivo por processo
///
/// Por simplicidade, usamos uma tabela global por enquanto.
/// TODO: Mover para estrutura por-processo
static FILE_HANDLES: Spinlock<BTreeMap<u32, FileHandle>> = Spinlock::new(BTreeMap::new());
static NEXT_HANDLE: Spinlock<u32> = Spinlock::new(3); // 0,1,2 reservados para stdin/stdout/stderr

/// Aloca um novo handle
pub fn alloc_handle(handle: FileHandle) -> u32 {
    let mut next = NEXT_HANDLE.lock();
    let id = *next;
    *next = next.wrapping_add(1);
    if *next < 3 {
        *next = 3; // Pular reservados
    }

    FILE_HANDLES.lock().insert(id, handle);
    id
}

/// Obtém um handle (imutável)
pub fn get_handle(id: u32) -> Option<FileHandle> {
    FILE_HANDLES.lock().get(&id).map(|h| FileHandle {
        path: h.path.clone(),
        file_type: h.file_type,
        flags: h.flags,
        offset: h.offset,
        size: h.size,
        first_cluster: h.first_cluster,
        dir_index: h.dir_index,
    })
}

/// Atualiza offset de um handle
pub fn update_offset(id: u32, new_offset: u64) -> bool {
    if let Some(handle) = FILE_HANDLES.lock().get_mut(&id) {
        handle.offset = new_offset;
        true
    } else {
        false
    }
}

/// Atualiza dir_index de um handle
pub fn update_dir_index(id: u32, new_index: usize) -> bool {
    if let Some(handle) = FILE_HANDLES.lock().get_mut(&id) {
        handle.dir_index = new_index;
        true
    } else {
        false
    }
}

/// Fecha um handle
pub fn close_handle(id: u32) -> bool {
    FILE_HANDLES.lock().remove(&id).is_some()
}

/// Lista todos os handles (para debug)
pub fn list_handles() -> Vec<(u32, String)> {
    FILE_HANDLES
        .lock()
        .iter()
        .map(|(id, h)| (*id, h.path.clone()))
        .collect()
}
