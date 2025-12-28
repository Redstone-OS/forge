//! Estruturas ELF para loading de executáveis

/// Magic number ELF
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// Classe ELF
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ElfClass {
    None = 0,
    Elf32 = 1,
    Elf64 = 2,
}

/// Tipo de arquivo ELF
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum ElfType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    SharedObject = 3,
    Core = 4,
}

/// Header ELF64
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Header {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub _pad: [u8; 7],
    pub elf_type: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// Tipo de program header
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum PhType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4,
    Phdr = 6,
    Tls = 7,
}

/// Program Header ELF64
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// Flags de program header
pub const PF_X: u32 = 1; // Executável
pub const PF_W: u32 = 2; // Escrevível
pub const PF_R: u32 = 4; // Legível

impl Elf64Header {
    /// Verifica se é ELF válido
    pub fn is_valid(&self) -> bool {
        self.magic == ELF_MAGIC && self.class == ElfClass::Elf64 as u8
    }
}
