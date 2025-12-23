//! Carregador de Executáveis ELF 64-bit.
//!
//! Responsável por parsear o binário, alocar memória virtual para o processo,
//! copiar os segmentos e preparar o Entry Point.

use crate::mm::pmm::FRAME_ALLOCATOR;
use crate::mm::vmm::{self, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::sys::Errno;
use alloc::vec::Vec;

// Constantes ELF
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const PT_LOAD: u32 = 1;
const PF_W: u32 = 2; // Flag Write
const PF_X: u32 = 1; // Flag Exec

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ElfHeader {
    magic: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    osabi: u8,
    abiversion: u8,
    pad: [u8; 7],
    typ: u16,
    machine: u16,
    version2: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ProgramHeader {
    typ: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

/// Carrega um binário ELF no espaço de endereçamento atual.
/// Retorna o Entry Point virtual ou Erro.
///
/// # Safety
/// Assume que o VMM atual é o do processo alvo.
pub unsafe fn load(data: &[u8]) -> Result<u64, Errno> {
    // 1. Validar Header
    if data.len() < size_of::<ElfHeader>() {
        return Err(Errno::ENOEXEC);
    }
    let header = &*(data.as_ptr() as *const ElfHeader);

    if header.magic != ELF_MAGIC || header.class != 2
    /* 64-bit */
    {
        return Err(Errno::ENOEXEC);
    }

    // 2. Iterar Program Headers
    let ph_offset = header.phoff as usize;
    let ph_size = header.phentsize as usize;
    let ph_count = header.phnum as usize;

    for i in 0..ph_count {
        let offset = ph_offset + i * ph_size;
        if offset + size_of::<ProgramHeader>() > data.len() {
            return Err(Errno::ENOEXEC);
        }

        let ph = &*(data.as_ptr().add(offset) as *const ProgramHeader);

        if ph.typ == PT_LOAD {
            if ph.memsz == 0 {
                continue;
            }

            // Calcular flags de página
            let mut page_flags = PAGE_PRESENT | PAGE_USER;
            if ph.flags & PF_W != 0 {
                page_flags |= PAGE_WRITABLE;
            }
            // if ph.flags & PF_X == 0 { page_flags |= PAGE_NO_EXEC; } // Futuro

            // Alocar e Mapear memória
            let start_addr = ph.vaddr;
            let end_addr = start_addr + ph.memsz;

            let start_page = start_addr & !0xFFF;
            let end_page = (end_addr + 0xFFF) & !0xFFF;

            // TODO: VMM deveria ter função map_range
            let mut curr = start_page;
            while curr < end_page {
                // Se já estiver mapeado (ex: overlap), ignorar ou erro?
                // Vamos assumir espaço limpo.
                let frame = FRAME_ALLOCATOR
                    .lock()
                    .allocate_frame()
                    .ok_or(Errno::ENOMEM)?;
                vmm::map_page(curr, frame.addr, page_flags);

                // Zerar memória (BSS requirement)
                let ptr = curr as *mut u8;
                core::ptr::write_bytes(ptr, 0, 4096);

                curr += 4096;
            }

            // Copiar dados do arquivo
            if ph.filesz > 0 {
                let dest = start_addr as *mut u8;
                let src = &data[ph.offset as usize..(ph.offset + ph.filesz) as usize];
                core::ptr::copy_nonoverlapping(src.as_ptr(), dest, src.len());
            }
        }
    }

    Ok(header.entry)
}

use core::mem::size_of;
