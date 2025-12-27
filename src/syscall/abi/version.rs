//! # ABI Versioning
//!
//! Versionamento para compatibilidade de binários.

/// Versão atual da ABI de syscalls
pub const ABI_VERSION: u32 = 1;

/// Magic number para binários Redstone
pub const REDSTONE_MAGIC: u32 = 0x5244_5354; // "RDST"

/// Header de binário userland
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BinaryHeader {
    /// Magic (deve ser REDSTONE_MAGIC)
    pub magic: u32,
    /// Versão da ABI usada
    pub abi_version: u32,
    /// Versão mínima do kernel requerida
    pub min_kernel: u32,
    /// Flags do binário
    pub flags: u32,
}

impl BinaryHeader {
    /// Verifica se o header é válido
    pub fn is_valid(&self) -> bool {
        self.magic == REDSTONE_MAGIC && self.abi_version <= ABI_VERSION
    }
}

/// Verifica compatibilidade de versão
pub fn check_compat(requested: u32) -> bool {
    requested <= ABI_VERSION
}
