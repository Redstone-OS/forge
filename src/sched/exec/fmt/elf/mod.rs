//! ELF Loader

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::VirtAddr;
use crate::sys::{KernelError, KernelResult};

mod structs;
use crate::mm::aspace::vma::{MemoryIntent, Protection, VmaFlags};
use crate::mm::aspace::{ASpaceError, AddressSpace};
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
            crate::ktrace!("(ELF) Segmento LOAD: vaddr=", phdr.p_vaddr);
            crate::ktrace!("(ELF) memsz=", phdr.p_memsz);
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

            let map_result = aspace_arc.lock().map_region(
                Some(start_vaddr),
                mem_size,
                prot,
                VmaFlags::empty(),
                intent,
            );

            match map_result {
                Ok(_) => {
                    crate::ktrace!("(ELF) VMA registrada:", start_vaddr.as_u64());
                }
                Err(ASpaceError::RegionOverlap) => {
                    // Sobreposição detectada (segmentos adjacentes compartilhando página)
                    // Vamos tentar fazer merge das permissões na VMA existente
                    crate::kwarn!("(ELF) Sobreposicao detectada. Tentando mesclar...");

                    let mut aspace = aspace_arc.lock();
                    if let Some(mut existing_vma) = aspace.find_vma(start_vaddr) {
                        // Atualizar permissões (Union)
                        // VMA struct é retornada por find_vma (clone).
                        // Precisamos atualizar a lista de VMAs.
                        // Mas, por enquanto, assumimos que se sobrepôs, a página anterior já existe.
                        // Vamos apenas garantir que a página física tenha permissão RWX se necessário no passo 3.
                        crate::kwarn!(
                            "(ELF) Mesclagem assumida. VMA existente:",
                            existing_vma.start.as_u64()
                        );
                    } else {
                        crate::kerror!("(ELF) Erro: Regiao sobreposta mas VMA nao encontrada!");
                        return Err(KernelError::OutOfMemory);
                    }
                }
                Err(e) => {
                    crate::kerror!("(ELF) Falha fatal ao registrar VMA:", e as u64);
                    return Err(KernelError::OutOfMemory);
                }
            }

            // 3. Alocar e mapear páginas físicas (Manual Load via HHDM)
            let start_page = phdr.p_vaddr & !(FRAME_SIZE - 1);
            let end_page = (phdr.p_vaddr + phdr.p_memsz + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
            let pages = (end_page - start_page) / FRAME_SIZE;

            let target_cr3 = aspace_arc.lock().cr3();
            let mut pmm = FRAME_ALLOCATOR.lock();
            let mut vmm_flags = MapFlags::PRESENT | MapFlags::USER | MapFlags::WRITABLE;

            if phdr.p_flags & 0x1 != 0 {
                vmm_flags |= MapFlags::EXECUTABLE;
            }

            for page_idx in 0..pages {
                let vaddr = start_page + page_idx * FRAME_SIZE;

                // Verificar se já está mapeado no alvo
                if crate::mm::vmm::mapper::translate_addr_in_p4(target_cr3, vaddr).is_none() {
                    if let Some(frame) = pmm.allocate_frame() {
                        unsafe {
                            crate::mm::vmm::mapper::map_page_in_target_p4(
                                target_cr3,
                                vaddr,
                                frame.as_u64(),
                                vmm_flags,
                                &mut *pmm,
                            )
                            .expect("(ELF) Erro ao mapear página");

                            // Zerar página NOVA via HHDM
                            core::ptr::write_bytes(
                                crate::mm::addr::phys_to_virt::<u8>(frame.as_u64()),
                                0,
                                FRAME_SIZE as usize,
                            );
                        }
                    }
                }
            }

            // 4. Copiar dados via HHDM para os frames do AddressSpace alvo
            let file_size = phdr.p_filesz as usize;
            if file_size > 0 {
                let mut bytes_copied = 0usize;
                let file_offset = phdr.p_offset as usize;
                let segment_data = &data[file_offset..file_offset + file_size];

                while bytes_copied < file_size {
                    let vaddr = phdr.p_vaddr + bytes_copied as u64;
                    let page_offset = vaddr % FRAME_SIZE;
                    let bytes_to_copy = core::cmp::min(
                        file_size - bytes_copied,
                        (FRAME_SIZE - page_offset) as usize,
                    );

                    // Achar frame físico correspondente no alvo
                    if let Some(phys) =
                        crate::mm::vmm::mapper::translate_addr_in_p4(target_cr3, vaddr)
                    {
                        unsafe {
                            let dst = crate::mm::addr::phys_to_virt::<u8>(phys & !0xFFF)
                                .add(page_offset as usize);
                            core::ptr::copy_nonoverlapping(
                                segment_data.as_ptr().add(bytes_copied),
                                dst,
                                bytes_to_copy,
                            );
                        }
                    } else {
                        panic!("(ELF) Erro fatal: página do segmento não mapeada!");
                    }

                    bytes_copied += bytes_to_copy;
                }
            }
        }
    }

    crate::ktrace!("(ELF) Carregado com sucesso. Entrada:", ehdr.e_entry);
    Ok(VirtAddr::new(ehdr.e_entry))
}
