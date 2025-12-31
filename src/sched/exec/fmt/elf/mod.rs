//! ELF Loader

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::VirtAddr;
use crate::sys::{KernelError, KernelResult};

mod structs;
use structs::*;

/// Carrega um binário ELF na memória
pub fn load_binary(data: &[u8]) -> KernelResult<VirtAddr> {
    // 1. Validar Magic Header (\x7FELF)
    if data.len() < 64 || &data[0..4] != b"\x7fELF" {
        crate::kerror!("(ELF) Invalid Magic");
        return Err(KernelError::InvalidArgument);
    }

    // Cast para Header
    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64_Ehdr) };

    // Validar arquitetura (x86_64 = 0x3E = 62)
    if ehdr.e_machine != 62 {
        crate::kerror!("(ELF) Invalid Arch:", ehdr.e_machine as u64);
        return Err(KernelError::InvalidArgument);
    }

    // Validar tipo (EXEC = 2, DYN = 3)
    if ehdr.e_type != ET_EXEC && ehdr.e_type != ET_DYN {
        crate::kerror!("(ELF) Invalid Type (Not EXEC/DYN):", ehdr.e_type as u64);
        return Err(KernelError::InvalidArgument);
    }

    crate::ktrace!("(ELF) Entry Point:", ehdr.e_entry);

    let ph_offset = ehdr.e_phoff as usize;
    let ph_num = ehdr.e_phnum as usize;
    let ph_size = ehdr.e_phentsize as usize;

    // Validar limites
    if ph_offset + ph_num * ph_size > data.len() {
        return Err(KernelError::InvalidArgument);
    }

    // Iterar Program Headers
    for i in 0..ph_num {
        let offset = ph_offset + i * ph_size;
        let phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64_Phdr) };

        if phdr.p_type == PT_LOAD {
            // Flags de mapeamento
            let mut flags = MapFlags::PRESENT | MapFlags::USER;
            if phdr.p_flags & PF_W != 0 {
                flags |= MapFlags::WRITABLE;
            }
            if phdr.p_flags & PF_X != 0 {
                flags |= MapFlags::EXECUTABLE;
            }

            crate::ktrace!("(ELF) MapFlags:", flags.bits() as u64);

            // Alocar e mapear páginas
            let start_page = phdr.p_vaddr & !(FRAME_SIZE - 1);
            let end_page = (phdr.p_vaddr + phdr.p_memsz + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);

            // crate::ktrace!("(ELF) start_page:", start_page);
            // crate::ktrace!("(ELF) end_page:", end_page);
            // crate::ktrace!("(ELF) FRAME_SIZE:", FRAME_SIZE);

            if FRAME_SIZE == 0 {
                crate::kerror!("(ELF) PANIC: FRAME_SIZE is 0!");
                return Err(KernelError::InvalidArgument);
            }

            let pages = (end_page - start_page) / FRAME_SIZE;

            let mut pmm = FRAME_ALLOCATOR.lock();

            for page_idx in 0..pages {
                let vaddr = start_page + page_idx * FRAME_SIZE;

                // CORREÇÃO: Verificar se a página já está mapeada
                // Se já está mapeada (por um segment anterior), NÃO re-alocar!
                // Isso resolve o problema de segmentos que compartilham a mesma página.
                if crate::mm::vmm::translate_addr(vaddr).is_some() {
                    // Página já mapeada - apenas pular, os dados serão copiados depois
                    continue;
                }

                // Página não mapeada - alocar novo frame
                if let Some(frame) = pmm.allocate_frame() {
                    let frame_phys = frame.as_u64();

                    // FORÇAR WRITABLE para poder zerar e copiar
                    let effective_flags = flags | MapFlags::WRITABLE;

                    if let Err(_e) =
                        map_page_with_pmm(vaddr, frame_phys, effective_flags, &mut *pmm)
                    {
                        crate::kerror!("(ELF) Map failed:", vaddr);
                        return Err(KernelError::OutOfMemory);
                    }

                    // Zera APENAS páginas recém-alocadas
                    unsafe {
                        let ptr = vaddr as *mut u8;
                        for i in 0..FRAME_SIZE as usize {
                            ptr.add(i).write_volatile(0);
                        }
                    }
                } else {
                    crate::kerror!("(ELF) Alloc failed OOM");
                    return Err(KernelError::OutOfMemory);
                }
            }

            // Copiar dados do arquivo para memória
            // IMPORTANTE: p_filesz pode ser menor que p_memsz (BSS)
            // A memória já foi zerada acima, então BSS já está limpo.
            let file_offset = phdr.p_offset as usize;
            let file_size = phdr.p_filesz as usize;

            if file_size > 0 {
                let dest = phdr.p_vaddr as *mut u8;

                // Validar bounds do arquivo
                if file_offset + file_size > data.len() {
                    crate::kerror!("(ELF) Segment out of bounds");
                    return Err(KernelError::InvalidArgument);
                }

                unsafe {
                    // Use Manual Copy to avoid intrinsics
                    for i in 0..file_size {
                        let b = data[file_offset + i];
                        dest.add(i).write_volatile(b);
                    }
                }
            }
        }
    }

    crate::ktrace!("(ELF) Loaded successfully. Entry:", ehdr.e_entry);
    Ok(VirtAddr::new(ehdr.e_entry))
}
