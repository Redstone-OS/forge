//! ELF Loader

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::VirtAddr;
use crate::sys::{KernelError, KernelResult};

mod structs;
use crate::mm::aspace::vma::{MemoryIntent, Protection, VmaFlags};
use crate::mm::aspace::AddressSpace;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use structs::*;

/// Carrega um binário ELF na memória de um AddressSpace
pub fn load_binary(
    data: &[u8],
    aspace_arc: &Arc<Spinlock<AddressSpace>>,
) -> KernelResult<VirtAddr> {
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

    let ph_offset = ehdr.e_phoff as usize;
    let ph_num = ehdr.e_phnum as usize;
    let ph_size = ehdr.e_phentsize as usize;

    // Iterar Program Headers
    for i in 0..ph_num {
        let offset = ph_offset + i * ph_size;
        let phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64_Phdr) };

        if phdr.p_type == PT_LOAD {
            // 1. Determinar Proteções e Intenção
            let mut prot = Protection::READ;
            if phdr.p_flags & PF_W != 0 {
                prot = Protection::RW;
            }
            if phdr.p_flags & PF_X != 0 {
                prot = if phdr.p_flags & PF_W != 0 {
                    Protection::RWX
                } else {
                    Protection::RX
                };
            }

            let intent = if phdr.p_flags & PF_X != 0 {
                MemoryIntent::Code
            } else if phdr.p_flags & PF_W != 0 {
                MemoryIntent::Data
            } else {
                MemoryIntent::FileReadOnly
            };

            // 2. Registrar VMA no AddressSpace
            let start_vaddr = VirtAddr::new(phdr.p_vaddr);
            let mem_size = phdr.p_memsz as usize;

            {
                let mut aspace = aspace_arc.lock();
                aspace
                    .map_region(Some(start_vaddr), mem_size, prot, VmaFlags::empty(), intent)
                    .expect("(ELF) Falha ao registrar VMA");
            }

            // 3. Alocar e mapear páginas físicas (Manual Load para agora)
            // Futuramente: isso será feito via Page Fault (Lazy Load)
            let start_page = phdr.p_vaddr & !(FRAME_SIZE - 1);
            let end_page = (phdr.p_vaddr + phdr.p_memsz + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
            let pages = (end_page - start_page) / FRAME_SIZE;

            let mut pmm = FRAME_ALLOCATOR.lock();
            let mut vmm_flags = MapFlags::PRESENT | MapFlags::USER | MapFlags::WRITABLE; // W p/ carregar

            for page_idx in 0..pages {
                let vaddr = start_page + page_idx * FRAME_SIZE;
                if let Some(frame) = pmm.allocate_frame() {
                    unsafe {
                        map_page_with_pmm(vaddr, frame.as_u64(), vmm_flags, &mut *pmm)
                            .expect("(ELF) Erro ao mapear página do segmento");

                        // Zerar página
                        core::ptr::write_bytes(vaddr as *mut u8, 0, FRAME_SIZE as usize);
                    }
                }
            }

            // 4. Copiar dados
            let file_size = phdr.p_filesz as usize;
            if file_size > 0 {
                let file_offset = phdr.p_offset as usize;
                let dest = phdr.p_vaddr as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(data.as_ptr().add(file_offset), dest, file_size);
                }
            }

            // 5. Ajustar Flags Finais (Efetivar RX se necessário)
            // TODO: Atualizar flags da page table para remover WRITABLE se original não tinha
        }
    }

    crate::ktrace!("(ELF) Loaded successfully. Entry:", ehdr.e_entry);
    Ok(VirtAddr::new(ehdr.e_entry))
}
