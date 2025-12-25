//! Carregador de Executáveis ELF 64-bit.
//!
//! Responsável por parsear o binário, alocar memória virtual para o processo,
//! copiar os segmentos e preparar o Entry Point.

use crate::mm::pmm::FRAME_ALLOCATOR;
use crate::mm::vmm::{self, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::sys::Errno;

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
            crate::ktrace!(
                "(Elf) PT_LOAD: vaddr={:#x} memsz={} filesz={}",
                ph.vaddr,
                ph.memsz,
                ph.filesz
            );

            if ph.memsz == 0 {
                continue;
            }

            // Verificar se vaddr é válido (não nulo para userspace)
            // if ph.vaddr == 0 {
            //     crate::kwarn!("[ELF] Skipping segment with vaddr=0");
            //     continue;
            // }
            // NOTA: Se o binário foi linkado em 0, precisamos carregar.
            // O init accessou 0x247, que está nesse segmento.

            // Calcular flags de página - durante carregamento SEMPRE writable
            // TODO: depois ajustar permissões corretas (read-only para .text etc)
            let page_flags = PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE;
            // if ph.flags & PF_X == 0 { page_flags |= PAGE_NO_EXEC; } // Futuro

            // Alocar e Mapear memória
            let start_addr = ph.vaddr;
            let end_addr = start_addr + ph.memsz;

            let start_page = start_addr & !0xFFF;
            let end_page = (end_addr + 0xFFF) & !0xFFF;

            crate::ktrace!("(Elf) Mapeando páginas {:#x} - {:#x}", start_page, end_page);

            // TODO: VMM deveria ter função map_range
            let mut curr = start_page;
            while curr < end_page {
                // Verificar se a página já está mapeada (overlap de segmentos)
                if let Some((_phys, flags)) = vmm::translate_addr_with_flags(curr) {
                    // Se já estiver mapeada como USER, é um overlap legítimo de segmentos ELF.
                    if flags & vmm::PAGE_USER != 0 {
                        crate::ktrace!("(Elf) Slot {:#x} já mapeado como USER (overlap ELF)", curr);
                        // IMPORTANTE: Não zerar a página se já existe e é USER!
                        curr += 4096;
                        continue;
                    }
                    // Se for mapeamento de KERNEL (ex: identity map), ignoramos e mapeamos por cima (com USER).
                }

                // Se não está mapeado, alocar novo frame
                let frame = FRAME_ALLOCATOR
                    .lock()
                    .allocate_frame()
                    .ok_or(Errno::ENOMEM)?;

                crate::ktrace!(
                    "(Elf) map_page: virt={:#x} phys={:#x} flags={:#x}",
                    curr,
                    frame.addr,
                    page_flags
                );
                vmm::map_page(curr, frame.addr, page_flags);

                // TLB flush para garantir que o mapeamento está visível
                core::arch::asm!("invlpg [{0}]", in(reg) curr, options(nostack, preserves_flags));

                // Zerar memória (BSS requirement) - loop manual para evitar write_bytes
                let ptr = curr as *mut u8;
                for j in 0..4096 {
                    core::ptr::write_volatile(ptr.add(j), 0);
                }

                curr += 4096;
            }
            crate::kinfo!("[ELF] Páginas mapeadas e zeradas");

            // Copiar dados do arquivo - loop manual para evitar copy_nonoverlapping
            if ph.filesz > 0 {
                crate::ktrace!("(Elf) Copiando {} bytes para {:#x}", ph.filesz, start_addr);
                let dest = start_addr as *mut u8;
                let src_offset = ph.offset as usize;
                let src_len = ph.filesz as usize;

                for j in 0..src_len {
                    let byte = data[src_offset + j];
                    core::ptr::write_volatile(dest.add(j), byte);
                }
            }
        }
    }

    Ok(header.entry)
}

use core::mem::size_of;
