//! # Estruturas ELF64
//!
//! Definições de estruturas e constantes do formato ELF64.

#![allow(dead_code)]
#![allow(non_camel_case_types)]

// =============================================================================
// TIPOS ELF
// =============================================================================

/// Arquivo Executável (static linked)
pub const ET_EXEC: u16 = 2;

/// Arquivo Dinâmico (PIE ou shared library)
pub const ET_DYN: u16 = 3;

// =============================================================================
// TIPOS DE SEGMENTO (Program Header)
// =============================================================================

/// Segmento NULL (ignorado)
pub const PT_NULL: u32 = 0;

/// Segmento Carregável
pub const PT_LOAD: u32 = 1;

/// Informações de Dynamic Linking
pub const PT_DYNAMIC: u32 = 2;

/// Caminho do Interpretador (ld.so)
pub const PT_INTERP: u32 = 3;

/// Notas auxiliares
pub const PT_NOTE: u32 = 4;

/// Program Header Table
pub const PT_PHDR: u32 = 6;

/// Thread-Local Storage template
pub const PT_TLS: u32 = 7;

// =============================================================================
// FLAGS DE SEGMENTO
// =============================================================================

/// Permissão de Execução
pub const PF_X: u32 = 0x1;

/// Permissão de Escrita
pub const PF_W: u32 = 0x2;

/// Permissão de Leitura
pub const PF_R: u32 = 0x4;

// =============================================================================
// ARQUITETURAS
// =============================================================================

/// x86_64 / AMD64
pub const EM_X86_64: u16 = 62;

// =============================================================================
// ESTRUTURAS
// =============================================================================

/// Cabeçalho ELF64 (Elf64_Ehdr)
///
/// Primeiros 64 bytes de todo arquivo ELF64.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    /// Identificação: magic, class, endian, version, OS/ABI
    pub e_ident: [u8; 16],
    /// Tipo do arquivo (ET_EXEC, ET_DYN, etc.)
    pub e_type: u16,
    /// Arquitetura alvo (EM_X86_64, etc.)
    pub e_machine: u16,
    /// Versão ELF (sempre 1)
    pub e_version: u32,
    /// Endereço de entrada (virtual)
    pub e_entry: u64,
    /// Offset do Program Header Table
    pub e_phoff: u64,
    /// Offset do Section Header Table
    pub e_shoff: u64,
    /// Flags específicas da arquitetura
    pub e_flags: u32,
    /// Tamanho deste header
    pub e_ehsize: u16,
    /// Tamanho de cada Program Header
    pub e_phentsize: u16,
    /// Número de Program Headers
    pub e_phnum: u16,
    /// Tamanho de cada Section Header
    pub e_shentsize: u16,
    /// Número de Section Headers
    pub e_shnum: u16,
    /// Índice da seção de strings de nomes
    pub e_shstrndx: u16,
}

/// Cabeçalho de Programa ELF64 (Elf64_Phdr)
///
/// Descreve um segmento a ser carregado em memória.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64ProgramHeader {
    /// Tipo do segmento (PT_LOAD, PT_INTERP, etc.)
    pub p_type: u32,
    /// Flags de permissão (PF_R, PF_W, PF_X)
    pub p_flags: u32,
    /// Offset no arquivo
    pub p_offset: u64,
    /// Endereço virtual de carga
    pub p_vaddr: u64,
    /// Endereço físico (geralmente ignorado)
    pub p_paddr: u64,
    /// Tamanho no arquivo
    pub p_filesz: u64,
    /// Tamanho em memória (pode ser maior que filesz para BSS)
    pub p_memsz: u64,
    /// Alinhamento do segmento
    pub p_align: u64,
}

impl Elf64Header {
    /// Tamanho esperado do header ELF64
    pub const SIZE: usize = 64;

    /// Valida o magic number ELF
    pub fn is_valid_magic(&self) -> bool {
        &self.e_ident[0..4] == b"\x7fELF"
    }

    /// Verifica se é ELF64 (class 2)
    pub fn is_64bit(&self) -> bool {
        self.e_ident[4] == 2
    }

    /// Verifica se é little-endian
    pub fn is_little_endian(&self) -> bool {
        self.e_ident[5] == 1
    }
}

impl Elf64ProgramHeader {
    /// Tamanho esperado do program header
    pub const SIZE: usize = 56;
}
