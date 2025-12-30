#![allow(dead_code)]
#![allow(non_camel_case_types)]
//! Estruturas ELF64

/// Arquivo Executável
pub const ET_EXEC: u16 = 2;
/// Arquivo Dinâmico (PIE)
pub const ET_DYN: u16 = 3;

/// Segmento Carregável
pub const PT_LOAD: u32 = 1;

/// Permissão de Execução
pub const PF_X: u32 = 1;
/// Permissão de Escrita
pub const PF_W: u32 = 2;
/// Permissão de Leitura
pub const PF_R: u32 = 4;

/// Cabeçalho ELF64
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// Cabeçalho de Programa ELF64 (Program Header)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}
