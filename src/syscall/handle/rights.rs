//! # Handle Rights
//!
//! Capabilities granulares para handles.

use crate::bitflags;

bitflags! {
    /// Direitos de um handle
    pub struct HandleRights: u64 {
        // === Básicos ===
        const READ       = 1 << 0;
        const WRITE      = 1 << 1;
        const EXEC       = 1 << 2;

        // === Handle ops ===
        const DUP        = 1 << 8;
        const TRANSFER   = 1 << 9;
        const CLOSE      = 1 << 10;

        // === Memory ops ===
        const MAP        = 1 << 16;
        const SHARE      = 1 << 17;

        // === Process ops ===
        const SIGNAL     = 1 << 24;
        const WAIT       = 1 << 25;
        const DEBUG      = 1 << 26;

        // === File ops ===
        const SEEK       = 1 << 32;
        const STAT       = 1 << 33;
        const TRUNCATE   = 1 << 34;

        // === Directory ops ===
        const READDIR    = 1 << 40;
        const CREATE     = 1 << 41;
        const DELETE     = 1 << 42;

        // === System ops ===
        const MOUNT      = 1 << 48;
        const ADMIN      = 1 << 63;
    }
}

impl HandleRights {
    /// Rights padrão para arquivo de leitura
    pub const FILE_READ: Self = Self::READ
        .union(Self::SEEK)
        .union(Self::STAT)
        .union(Self::CLOSE);

    /// Rights padrão para arquivo de escrita
    pub const FILE_WRITE: Self = Self::WRITE
        .union(Self::SEEK)
        .union(Self::STAT)
        .union(Self::CLOSE);

    /// Rights padrão para arquivo RW
    pub const FILE_RW: Self = Self::FILE_READ.union(Self::FILE_WRITE);

    /// Rights padrão para diretório
    pub const DIRECTORY: Self = Self::READ
        .union(Self::READDIR)
        .union(Self::STAT)
        .union(Self::CLOSE);

    /// Verifica se pode reduzir para new_rights
    pub fn can_reduce_to(&self, new_rights: HandleRights) -> bool {
        self.contains(new_rights)
    }
}

/// Valida se operação é permitida
pub fn check_rights(current: HandleRights, required: HandleRights) -> bool {
    current.contains(required)
}
