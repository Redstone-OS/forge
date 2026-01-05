//! # ELF Parser
//!
//! Parsing puro de binários ELF64 sem efeitos colaterais.
//! Este módulo apenas interpreta os bytes, não aloca memória.

use super::structs::*;
use crate::rmm::addr::VirtAddr;
use crate::rmm::virt::aspace::vma::Protection;
use crate::sched::exec::error::ExecError;

/// Segmento parseado do ELF
///
/// Representa um PT_LOAD pronto para ser mapeado.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Endereço virtual onde carregar
    pub vaddr: VirtAddr,
    /// Tamanho em memória (inclui BSS)
    pub mem_size: usize,
    /// Tamanho no arquivo (dados a copiar)
    pub file_size: usize,
    /// Offset no arquivo fonte
    pub file_offset: usize,
    /// Proteção de memória
    pub protection: Protection,
    /// Flags originais do ELF
    pub flags: u32,
}

/// Resultado do parsing de um ELF
#[derive(Debug)]
pub struct ParsedElf<'a> {
    /// Endereço de entrada
    pub entry_point: VirtAddr,
    /// Segmentos a carregar
    pub segments: alloc::vec::Vec<Segment>,
    /// Referência aos dados originais
    pub data: &'a [u8],
}

/// Parseia um binário ELF64
///
/// # Argumentos
///
/// * `data` - Bytes do arquivo ELF
///
/// # Retorna
///
/// * `Ok(ParsedElf)` - Estrutura com entry point e segmentos
/// * `Err(ExecError)` - Se houver erro de parsing ou formato inválido
///
/// # Erros
///
/// - `InvalidElf` - Magic inválido ou não é ELF64
/// - `UnsupportedArch` - Não é x86_64
/// - `UnsupportedType` - É PIE (ET_DYN)
/// - `HeaderOutOfBounds` - Program header fora do arquivo
/// - `SegmentOutOfBounds` - Dados do segmento fora do arquivo
pub fn parse(data: &[u8]) -> Result<ParsedElf<'_>, ExecError> {
    // Validar tamanho mínimo
    if data.len() < Elf64Header::SIZE {
        return Err(ExecError::InvalidElf);
    }

    // Parsear header
    let header = unsafe { &*(data.as_ptr() as *const Elf64Header) };

    // Validar magic
    if !header.is_valid_magic() {
        return Err(ExecError::InvalidElf);
    }

    // Validar que é ELF64 little-endian
    if !header.is_64bit() || !header.is_little_endian() {
        return Err(ExecError::InvalidElf);
    }

    // Validar arquitetura (x86_64)
    if header.e_machine != EM_X86_64 {
        return Err(ExecError::UnsupportedArch);
    }

    // Rejeitar PIE/DYN explicitamente
    if header.e_type == ET_DYN {
        return Err(ExecError::UnsupportedType);
    }

    // Aceitar apenas ET_EXEC
    if header.e_type != ET_EXEC {
        return Err(ExecError::InvalidElf);
    }

    // Parsear program headers
    let ph_offset = header.e_phoff as usize;
    let ph_num = header.e_phnum as usize;
    let ph_size = header.e_phentsize as usize;

    // Validar bounds do program header table
    let ph_table_end = ph_offset.saturating_add(ph_num.saturating_mul(ph_size));
    if ph_table_end > data.len() {
        return Err(ExecError::HeaderOutOfBounds);
    }

    let mut segments = alloc::vec::Vec::new();

    for i in 0..ph_num {
        let offset = ph_offset + i * ph_size;

        // Safety: bounds já validados acima
        let phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64ProgramHeader) };

        // Ignorar segmentos não-LOAD
        if phdr.p_type != PT_LOAD {
            continue;
        }

        // Validar bounds do segmento no arquivo
        let seg_file_end = (phdr.p_offset as usize).saturating_add(phdr.p_filesz as usize);
        if seg_file_end > data.len() {
            return Err(ExecError::SegmentOutOfBounds);
        }

        // Determinar proteção
        let protection = flags_to_protection(phdr.p_flags);

        segments.push(Segment {
            vaddr: VirtAddr::new(phdr.p_vaddr),
            mem_size: phdr.p_memsz as usize,
            file_size: phdr.p_filesz as usize,
            file_offset: phdr.p_offset as usize,
            protection,
            flags: phdr.p_flags,
        });
    }

    Ok(ParsedElf {
        entry_point: VirtAddr::new(header.e_entry),
        segments,
        data,
    })
}

/// Converte flags ELF (PF_*) para Protection do VMA
fn flags_to_protection(flags: u32) -> Protection {
    let read = flags & PF_R != 0;
    let write = flags & PF_W != 0;
    let exec = flags & PF_X != 0;

    // Usar Protection::new() para construir
    Protection::new(read, write, exec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_to_protection() {
        assert_eq!(flags_to_protection(PF_R), Protection::RO);
        assert_eq!(flags_to_protection(PF_R | PF_W), Protection::RW);
        assert_eq!(flags_to_protection(PF_R | PF_X), Protection::RX);
        assert_eq!(flags_to_protection(PF_R | PF_W | PF_X), Protection::RWX);
    }
}
