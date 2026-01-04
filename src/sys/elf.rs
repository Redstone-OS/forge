//! # Estruturas ELF64
//!
//! Definições para parsing e loading de executáveis ELF64.
//!
//! ## Referência
//!
//! System V Application Binary Interface (ABI)
//! AMD64 Architecture Processor Supplement

// =============================================================================
// CONSTANTES
// =============================================================================

/// Magic number ELF: `[0x7F, 'E', 'L', 'F']`
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// Máquina x86-64
pub const EM_X86_64: u16 = 62;

// =============================================================================
// ENUMS
// =============================================================================

/// Classe ELF (32 ou 64 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElfClass {
    /// Inválido
    None = 0,
    /// 32-bit
    Elf32 = 1,
    /// 64-bit
    Elf64 = 2,
}

/// Endianness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElfEndian {
    /// Inválido
    None = 0,
    /// Little-endian
    Little = 1,
    /// Big-endian
    Big = 2,
}

/// Tipo de arquivo ELF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ElfType {
    /// Sem tipo
    None = 0,
    /// Relocatable (.o)
    Relocatable = 1,
    /// Executável
    Executable = 2,
    /// Shared object (.so)
    SharedObject = 3,
    /// Core dump
    Core = 4,
}

/// Tipo de Program Header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PhType {
    /// Ignorar
    Null = 0,
    /// Segmento carregável
    Load = 1,
    /// Informações de linkagem dinâmica
    Dynamic = 2,
    /// Caminho do interpretador
    Interp = 3,
    /// Informações auxiliares
    Note = 4,
    /// Program Header Table
    Phdr = 6,
    /// Thread Local Storage
    Tls = 7,
    /// GNU Stack
    GnuStack = 0x6474_e551,
    /// GNU Relro
    GnuRelro = 0x6474_e552,
}

impl PhType {
    /// Cria a partir de valor u32.
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Null,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Interp,
            4 => Self::Note,
            6 => Self::Phdr,
            7 => Self::Tls,
            0x6474_e551 => Self::GnuStack,
            0x6474_e552 => Self::GnuRelro,
            _ => Self::Null,
        }
    }
}

// =============================================================================
// ESTRUTURAS
// =============================================================================

/// Header ELF64 (64 bytes).
///
/// Localizado no início do arquivo ELF.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Header {
    /// Magic number `[0x7F, 'E', 'L', 'F']`
    pub magic: [u8; 4],
    /// Classe (32 ou 64 bits)
    pub class: u8,
    /// Endianness (1=little, 2=big)
    pub endian: u8,
    /// Versão do ELF (1)
    pub version: u8,
    /// OS/ABI (0=System V)
    pub os_abi: u8,
    /// Versão do ABI
    pub abi_version: u8,
    /// Padding
    pub _pad: [u8; 7],
    /// Tipo de arquivo (executável, shared, etc)
    pub elf_type: u16,
    /// Arquitetura (x86-64 = 62)
    pub machine: u16,
    /// Versão (1)
    pub version2: u32,
    /// Endereço de entrada (entry point)
    pub entry: u64,
    /// Offset da tabela de Program Headers
    pub phoff: u64,
    /// Offset da tabela de Section Headers
    pub shoff: u64,
    /// Flags específicas da arquitetura
    pub flags: u32,
    /// Tamanho deste header (64)
    pub ehsize: u16,
    /// Tamanho de cada Program Header
    pub phentsize: u16,
    /// Número de Program Headers
    pub phnum: u16,
    /// Tamanho de cada Section Header
    pub shentsize: u16,
    /// Número de Section Headers
    pub shnum: u16,
    /// Índice da seção de strings de nomes
    pub shstrndx: u16,
}

impl Elf64Header {
    /// Tamanho em bytes do header.
    pub const SIZE: usize = 64;

    /// Verifica se é ELF válido.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.magic == ELF_MAGIC
            && self.class == ElfClass::Elf64 as u8
            && self.endian == ElfEndian::Little as u8
    }

    /// Verifica se é executável.
    #[inline]
    pub fn is_executable(&self) -> bool {
        self.elf_type == ElfType::Executable as u16 || self.elf_type == ElfType::SharedObject as u16
    }

    /// Verifica se é para x86-64.
    #[inline]
    pub fn is_x86_64(&self) -> bool {
        self.machine == EM_X86_64
    }

    /// Retorna tipo do ELF.
    #[inline]
    pub fn elf_type(&self) -> ElfType {
        match self.elf_type {
            0 => ElfType::None,
            1 => ElfType::Relocatable,
            2 => ElfType::Executable,
            3 => ElfType::SharedObject,
            4 => ElfType::Core,
            _ => ElfType::None,
        }
    }
}

/// Program Header ELF64 (56 bytes).
///
/// Descreve um segmento a ser carregado em memória.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Phdr {
    /// Tipo do segmento
    pub p_type: u32,
    /// Flags (R/W/X)
    pub p_flags: u32,
    /// Offset no arquivo
    pub p_offset: u64,
    /// Endereço virtual
    pub p_vaddr: u64,
    /// Endereço físico (geralmente ignorado)
    pub p_paddr: u64,
    /// Tamanho no arquivo
    pub p_filesz: u64,
    /// Tamanho em memória (>= filesz para .bss)
    pub p_memsz: u64,
    /// Alinhamento
    pub p_align: u64,
}

impl Elf64Phdr {
    /// Tamanho em bytes do header.
    pub const SIZE: usize = 56;

    /// Retorna tipo do segmento.
    #[inline]
    pub fn segment_type(&self) -> PhType {
        PhType::from_u32(self.p_type)
    }

    /// Verifica se é carregável.
    #[inline]
    pub fn is_load(&self) -> bool {
        self.p_type == PhType::Load as u32
    }

    /// Verifica se é executável.
    #[inline]
    pub fn is_executable(&self) -> bool {
        (self.p_flags & PF_X) != 0
    }

    /// Verifica se é escrevível.
    #[inline]
    pub fn is_writable(&self) -> bool {
        (self.p_flags & PF_W) != 0
    }

    /// Verifica se é legível.
    #[inline]
    pub fn is_readable(&self) -> bool {
        (self.p_flags & PF_R) != 0
    }

    /// Bytes que precisam ser zerados (.bss).
    #[inline]
    pub fn bss_size(&self) -> u64 {
        self.p_memsz.saturating_sub(self.p_filesz)
    }
}

// =============================================================================
// FLAGS
// =============================================================================

/// Flag: Executável
pub const PF_X: u32 = 1;
/// Flag: Escrevível
pub const PF_W: u32 = 2;
/// Flag: Legível
pub const PF_R: u32 = 4;

// =============================================================================
// SECTION HEADER (para referência futura)
// =============================================================================

/// Section Header ELF64 (64 bytes).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Shdr {
    /// Offset do nome na tabela de strings
    pub sh_name: u32,
    /// Tipo da seção
    pub sh_type: u32,
    /// Flags
    pub sh_flags: u64,
    /// Endereço em memória
    pub sh_addr: u64,
    /// Offset no arquivo
    pub sh_offset: u64,
    /// Tamanho
    pub sh_size: u64,
    /// Link para outra seção
    pub sh_link: u32,
    /// Informação extra
    pub sh_info: u32,
    /// Alinhamento
    pub sh_addralign: u64,
    /// Tamanho de entrada (para tabelas)
    pub sh_entsize: u64,
}

impl Elf64Shdr {
    /// Tamanho em bytes.
    pub const SIZE: usize = 64;
}
