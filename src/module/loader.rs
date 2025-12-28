//! # Module Loader
//!
//! Carrega módulos ELF do sistema de arquivos.
//!
//! ## Responsabilidades
//! - Ler arquivo do VFS
//! - Parsear formato ELF
//! - Alocar páginas separadas para código (RX) e dados (RW)
//! - Resolver símbolos da Module ABI

use super::{LoadedModule, ModuleError};
use crate::fs::vfs::file::{File, FileOps, OpenFlags};
use alloc::vec::Vec;

/// Carregador de módulos ELF
#[allow(dead_code)]
pub struct ModuleLoader {
    /// Se deve verificar W^X (código nunca é escrevível)
    enforce_wx: bool,
}

impl ModuleLoader {
    /// Cria um novo loader
    pub const fn new() -> Self {
        Self { enforce_wx: true }
    }

    /// Carrega dados brutos do módulo do VFS
    pub fn load_from_vfs(&self, path: &str) -> Result<Vec<u8>, ModuleError> {
        use crate::fs::vfs::ROOT_VFS;

        let vfs = ROOT_VFS.lock();

        let node = vfs.lookup(path).map_err(|_| ModuleError::NotFound)?;

        // let handle = node.open().map_err(|_| ModuleError::InternalError)?;
        // WARNING: Unsafe File creation from reference. Node must live long enough.
        // Assuming single-threaded bootloader usage where node won't vanish.
        let handle = File::new(&node, OpenFlags(OpenFlags::READ));

        let size = node.size as usize;
        if size == 0 {
            return Err(ModuleError::InvalidFormat);
        }

        let mut buffer = Vec::with_capacity(size);
        unsafe {
            buffer.set_len(size);
        }

        handle
            .read(&mut buffer)
            .map_err(|_| ModuleError::InternalError)?;

        Ok(buffer)
    }

    /// Parseia ELF e carrega nas páginas do módulo
    pub fn parse_and_load(
        &self,
        elf_data: &[u8],
        module: &mut LoadedModule,
    ) -> Result<(), ModuleError> {
        // Verificar magic ELF
        if elf_data.len() < 4 || &elf_data[0..4] != b"\x7FELF" {
            return Err(ModuleError::InvalidFormat);
        }

        // Verificar que é 64-bit
        if elf_data.len() < 5 || elf_data[4] != 2 {
            return Err(ModuleError::InvalidFormat);
        }

        // Extrair entry point (offset 0x18 em ELF64)
        if elf_data.len() >= 0x20 {
            module.entry_point = u64::from_le_bytes([
                elf_data[0x18],
                elf_data[0x19],
                elf_data[0x1A],
                elf_data[0x1B],
                elf_data[0x1C],
                elf_data[0x1D],
                elf_data[0x1E],
                elf_data[0x1F],
            ]);
        }

        // Alocar páginas de código (1 página por 4KB de dados)
        let code_pages_needed = (elf_data.len() + 4095) / 4096;
        if code_pages_needed > module.limits.max_code_pages {
            return Err(ModuleError::LimitReached);
        }

        // Alocar páginas via PMM
        for _ in 0..code_pages_needed {
            match crate::mm::pmm::FRAME_ALLOCATOR.lock().allocate_frame() {
                Some(frame) => {
                    module.code_pages.push(frame.addr());
                }
                None => {
                    // Liberar páginas já alocadas
                    self.free_pages(module);
                    return Err(ModuleError::InternalError);
                }
            }
        }

        // TODO: Copiar código para as páginas
        // TODO: Parsear seções .data e .bss

        Ok(())
    }

    /// Libera páginas de um módulo
    pub fn free_pages(&self, module: &mut LoadedModule) {
        // TODO: Implementar liberação real quando API estiver estável
        // Por agora, apenas limpa as listas
        module.code_pages.clear();
        module.data_pages.clear();
    }
}
