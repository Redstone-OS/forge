//! ELF Loader
//!
//! Carrega binários ELF 64-bit na memória.

#![allow(dead_code)]

/// ELF Magic Number
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF Class
const ELFCLASS64: u8 = 2; // 64-bit

/// ELF Data Encoding
const ELFDATA2LSB: u8 = 1; // Little-endian

/// ELF Type
const ET_EXEC: u16 = 2; // Executable file

/// ELF Machine
const EM_X86_64: u16 = 0x3E; // AMD x86-64

/// Program Header Type
const PT_LOAD: u32 = 1; // Loadable segment

/// ELF Header (64-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    pub magic: [u8; 4],    // 0x7F, 'E', 'L', 'F'
    pub class: u8,         // 1=32-bit, 2=64-bit
    pub data: u8,          // 1=little-endian, 2=big-endian
    pub version: u8,       // ELF version (1)
    pub os_abi: u8,        // OS ABI
    pub abi_version: u8,   // ABI version
    pub _padding: [u8; 7], // Padding
    pub type_: u16,        // 1=relocatable, 2=executable, 3=shared, 4=core
    pub machine: u16,      // Architecture (0x3E=x86-64)
    pub version2: u32,     // ELF version (1)
    pub entry: u64,        // Entry point address
    pub phoff: u64,        // Program header table offset
    pub shoff: u64,        // Section header table offset
    pub flags: u32,        // Processor-specific flags
    pub ehsize: u16,       // ELF header size
    pub phentsize: u16,    // Program header entry size
    pub phnum: u16,        // Number of program headers
    pub shentsize: u16,    // Section header entry size
    pub shnum: u16,        // Number of section headers
    pub shstrndx: u16,     // Section header string table index
}

/// Program Header (64-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader {
    pub type_: u32,  // Segment type (1=LOAD)
    pub flags: u32,  // Segment flags
    pub offset: u64, // Offset in file
    pub vaddr: u64,  // Virtual address
    pub paddr: u64,  // Physical address (ignored)
    pub filesz: u64, // Size in file
    pub memsz: u64,  // Size in memory
    pub align: u64,  // Alignment
}

/// Erros de carregamento ELF
#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    NotElf64,
    NotLittleEndian,
    NotExecutable,
    WrongArchitecture,
    InvalidProgramHeader,
}

impl ElfError {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElfError::InvalidMagic => "Invalid ELF magic number",
            ElfError::NotElf64 => "Not a 64-bit ELF",
            ElfError::NotLittleEndian => "Not little-endian",
            ElfError::NotExecutable => "Not an executable",
            ElfError::WrongArchitecture => "Wrong architecture (not x86-64)",
            ElfError::InvalidProgramHeader => "Invalid program header",
        }
    }
}

/// Carrega ELF na memória
///
/// # Arguments
///
/// * `data` - Dados do arquivo ELF
///
/// # Returns
///
/// Entry point do binário carregado
///
/// # Safety
///
/// Esta função é unsafe porque:
/// - Escreve diretamente na memória em endereços especificados pelo ELF
/// - Não valida se os endereços são seguros
/// - Assume que há memória suficiente
pub unsafe fn load_elf(data: &[u8]) -> Result<u64, ElfError> {
    // Validar tamanho mínimo
    if data.len() < core::mem::size_of::<ElfHeader>() {
        return Err(ElfError::InvalidMagic);
    }

    // Ler header
    let header = &*(data.as_ptr() as *const ElfHeader);

    // Validar magic number
    if header.magic != ELF_MAGIC {
        return Err(ElfError::InvalidMagic);
    }

    // Validar 64-bit
    if header.class != ELFCLASS64 {
        return Err(ElfError::NotElf64);
    }

    // Validar little-endian
    if header.data != ELFDATA2LSB {
        return Err(ElfError::NotLittleEndian);
    }

    // Validar tipo executável
    if header.type_ != ET_EXEC {
        return Err(ElfError::NotExecutable);
    }

    // Validar arquitetura x86-64
    if header.machine != EM_X86_64 {
        return Err(ElfError::WrongArchitecture);
    }

    // Carregar program headers
    for i in 0..header.phnum {
        let phdr_offset = header.phoff as usize + (i as usize * header.phentsize as usize);

        if phdr_offset + core::mem::size_of::<ProgramHeader>() > data.len() {
            return Err(ElfError::InvalidProgramHeader);
        }

        let phdr = &*(data.as_ptr().add(phdr_offset) as *const ProgramHeader);

        // Apenas carregar segmentos PT_LOAD
        if phdr.type_ == PT_LOAD {
            // Validar offsets
            if phdr.offset as usize + phdr.filesz as usize > data.len() {
                return Err(ElfError::InvalidProgramHeader);
            }

            // Copiar dados do arquivo para memória
            let src = &data[phdr.offset as usize..(phdr.offset + phdr.filesz) as usize];
            let dst = core::slice::from_raw_parts_mut(phdr.vaddr as *mut u8, phdr.memsz as usize);

            // Copiar dados do arquivo
            dst[..phdr.filesz as usize].copy_from_slice(src);

            // Zerar BSS (se memsz > filesz)
            if phdr.memsz > phdr.filesz {
                dst[phdr.filesz as usize..].fill(0);
            }
        }
    }

    // Retornar entry point
    Ok(header.entry)
}

/// Valida se dados são um ELF válido sem carregar
pub fn validate_elf(data: &[u8]) -> Result<(), ElfError> {
    if data.len() < core::mem::size_of::<ElfHeader>() {
        return Err(ElfError::InvalidMagic);
    }

    let header = unsafe { &*(data.as_ptr() as *const ElfHeader) };

    if header.magic != ELF_MAGIC {
        return Err(ElfError::InvalidMagic);
    }

    if header.class != ELFCLASS64 {
        return Err(ElfError::NotElf64);
    }

    if header.data != ELFDATA2LSB {
        return Err(ElfError::NotLittleEndian);
    }

    if header.type_ != ET_EXEC {
        return Err(ElfError::NotExecutable);
    }

    if header.machine != EM_X86_64 {
        return Err(ElfError::WrongArchitecture);
    }

    Ok(())
}
