//! ABI estável para módulos

/// Versão da ABI
pub const ABI_VERSION: u32 = 1;

/// Magic number para validação
pub const MODULE_MAGIC: u32 = 0x4D4F4452; // "MODR"

/// Informações do módulo (header no binário)
#[repr(C)]
pub struct ModuleInfo {
    /// Magic number
    pub magic: u32,
    /// Versão da ABI
    pub abi_version: u32,
    /// Nome do módulo (null-terminated)
    pub name: [u8; 32],
    /// Versão do módulo
    pub version: u32,
    /// Flags
    pub flags: u32,
    /// Capabilities requisitadas (bitmask)
    pub required_caps: u64,
}

/// ABI de callbacks do módulo
#[repr(C)]
pub struct ModuleAbi {
    /// Chamado para inicializar
    pub init: Option<extern "C" fn() -> i32>,
    /// Chamado para cleanup
    pub cleanup: Option<extern "C" fn()>,
    /// Chamado por healthcheck
    pub health: Option<extern "C" fn() -> i32>,
}

impl ModuleInfo {
    /// Verifica se é válido
    pub fn is_valid(&self) -> bool {
        self.magic == MODULE_MAGIC && self.abi_version == ABI_VERSION
    }
}
