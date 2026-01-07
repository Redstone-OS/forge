//! # ELF Loader
//!
//! Carregamento de binários ELF64 em um Address Space.
//!
//! Este módulo usa o parser para interpretar o ELF e o RMM para
//! alocar e mapear memória no Address Space do processo alvo.

mod parser;
mod structs;

pub use parser::{parse, ParsedElf, Segment};
pub use structs::*;

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::aspace::vma::MemoryIntent;
use crate::rmm::virt::aspace::AddressSpace;
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::sched::exec::error::ExecError;
use crate::sync::Spinlock;
use alloc::sync::Arc;

/// Carrega um binário ELF em um Address Space
///
/// # Argumentos
///
/// * `data` - Bytes do arquivo ELF
/// * `aspace` - Address Space onde carregar
///
/// # Retorna
///
/// * `Ok(entry_point)` - Endereço de entrada do binário
/// * `Err(ExecError)` - Se houver falha no parsing ou mapeamento
pub fn load_binary(
    data: &[u8],
    aspace: &Arc<Spinlock<AddressSpace>>,
) -> Result<VirtAddr, ExecError> {
    // 1. Parsear ELF
    let elf = parser::parse(data)?;

    crate::ktrace!("(ELF) Parsed binary, entry:", elf.entry_point.as_u64());

    // 2. Obter CR3 do address space alvo
    let target_cr3 = aspace.lock().cr3();

    // 3. Obter PID do address space para alocação correta
    let owner_pid = aspace.lock().owner();

    // 4. Carregar cada segmento
    for segment in &elf.segments {
        load_segment(segment, data, aspace, target_cr3, owner_pid)?;
    }

    crate::ktrace!(
        "(ELF) Loaded successfully, entry:",
        elf.entry_point.as_u64()
    );
    Ok(elf.entry_point)
}

/// Carrega um único segmento no Address Space
fn load_segment(
    segment: &Segment,
    data: &[u8],
    aspace: &Arc<Spinlock<AddressSpace>>,
    target_cr3: u64,
    owner_pid: u32,
) -> Result<(), ExecError> {
    let page_size = PAGE_SIZE as u64;

    // Calcular range de páginas
    let start_page = segment.vaddr.as_u64() & !(page_size - 1);
    let end_addr = segment.vaddr.as_u64() + segment.mem_size as u64;
    let end_page = (end_addr + page_size - 1) & !(page_size - 1);
    let num_pages = ((end_page - start_page) / page_size) as usize;

    crate::ktrace!("(ELF) Loading segment at:", segment.vaddr.as_u64());
    crate::ktrace!("(ELF)   pages:", num_pages as u64);

    // Determinar intent baseado nas flags
    let intent = if segment.flags & structs::PF_X != 0 {
        MemoryIntent::Code
    } else if segment.flags & structs::PF_W != 0 {
        MemoryIntent::Data
    } else {
        MemoryIntent::Rodata // Read-only data
    };

    // 1. Registrar VMA no Address Space
    // Alinhar start e size a página
    let aligned_start = VirtAddr::new(start_page);
    let aligned_size = (num_pages * PAGE_SIZE) as usize;

    {
        let mut as_guard = aspace.lock();
        match as_guard.map_region(aligned_start, aligned_size, segment.protection, intent) {
            Ok(_) => {
                crate::ktrace!("(ELF)   VMA registered");
            }
            Err(crate::rmm::error::RmmError::InvalidAddress) => {
                // Sobreposição de segmentos adjacentes - OK
                crate::ktrace!("(ELF)   VMA overlap, merging");
            }
            Err(crate::rmm::error::RmmError::AlreadyMapped) => {
                // Overlap com VMA já existente - pode ser segmento adjacente
                crate::ktrace!("(ELF)   VMA overlap (already mapped)");
            }
            Err(e) => {
                crate::kerror!("(ELF)   VMA failed!");
                return Err(ExecError::MappingFailed);
            }
        }
    }

    // 2. Alocar e mapear páginas físicas
    for page_idx in 0..num_pages {
        let vaddr = start_page + (page_idx as u64) * page_size;

        // Verificar se já está mapeada (pode ser overlap de segmento anterior)
        if is_page_mapped(target_cr3, vaddr) {
            continue;
        }

        // Alocar frame zerado para userspace
        let frame = phys::alloc(
            FrameOwner::Process { pid: owner_pid },
            Zone::Normal,
            AllocFlags::ZERO,
        )
        .ok_or_else(|| {
            crate::kerror!("(ELF)   OOM at page", page_idx as u64);
            ExecError::OutOfMemory
        })?;

        // Mapear no address space alvo
        if let Err(e) = map_page_in_target(target_cr3, vaddr, frame.as_u64(), segment.protection) {
            crate::kerror!("(ELF)   Map failed at", vaddr);
            return Err(e);
        }
    }
    crate::ktrace!("(ELF)   Pages mapped");

    // 3. Copiar dados do segmento
    if segment.file_size > 0 {
        crate::ktrace!("(ELF)   Copying data, size:", segment.file_size as u64);
        copy_segment_data(segment, data, target_cr3)?;
        crate::ktrace!("(ELF)   Data copied");
    }

    crate::ktrace!("(ELF)   Segment done");
    Ok(())
}

/// Copia dados do ELF para as páginas mapeadas
fn copy_segment_data(segment: &Segment, data: &[u8], target_cr3: u64) -> Result<(), ExecError> {
    let page_size = PAGE_SIZE as u64;
    let segment_data = &data[segment.file_offset..segment.file_offset + segment.file_size];

    let mut bytes_copied = 0usize;

    while bytes_copied < segment.file_size {
        let vaddr = segment.vaddr.as_u64() + bytes_copied as u64;
        let page_offset = vaddr % page_size;
        let bytes_to_copy = core::cmp::min(
            segment.file_size - bytes_copied,
            (page_size - page_offset) as usize,
        );

        // Traduzir para endereço físico
        let phys = translate_in_target(target_cr3, vaddr).ok_or(ExecError::MappingFailed)?;

        unsafe {
            let dst = hhdm::phys_to_virt(phys & !(page_size - 1)) as *mut u8;
            let dst = dst.add(page_offset as usize);
            core::ptr::copy_nonoverlapping(
                segment_data.as_ptr().add(bytes_copied),
                dst,
                bytes_to_copy,
            );
        }

        bytes_copied += bytes_to_copy;
    }

    Ok(())
}

/// Verifica se uma página já está mapeada no address space alvo
fn is_page_mapped(target_cr3: u64, vaddr: u64) -> bool {
    translate_in_target(target_cr3, vaddr).is_some()
}

/// Mapeia uma página física no address space alvo
fn map_page_in_target(
    target_cr3: u64,
    vaddr: u64,
    phys_frame: u64,
    _protection: crate::rmm::virt::aspace::vma::Protection,
) -> Result<(), ExecError> {
    // Flags para tabelas intermediárias (sempre writable e user para permitir acesso)
    let table_flags: u64 = 0x7; // Present | Writable | User

    unsafe {
        let pml4_phys = target_cr3 & !0xFFF;
        let pml4 = hhdm::phys_to_virt(pml4_phys) as *mut u64;

        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;

        // Ensure PDPT exists
        if (*pml4.add(pml4_idx)) & 1 == 0 {
            let new_pdpt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pml4.add(pml4_idx) = new_pdpt.as_u64() | table_flags;
        }

        let pdpt_phys = (*pml4.add(pml4_idx)) & 0x000F_FFFF_FFFF_F000;
        let pdpt = hhdm::phys_to_virt(pdpt_phys) as *mut u64;

        // Ensure PD exists
        if (*pdpt.add(pdpt_idx)) & 1 == 0 {
            let new_pd = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pdpt.add(pdpt_idx) = new_pd.as_u64() | table_flags;
        }

        let pd_phys = (*pdpt.add(pdpt_idx)) & 0x000F_FFFF_FFFF_F000;
        let pd = hhdm::phys_to_virt(pd_phys) as *mut u64;

        // Ensure PT exists
        if (*pd.add(pd_idx)) & 1 == 0 {
            let new_pt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pd.add(pd_idx) = new_pt.as_u64() | table_flags;
        }

        let pt_phys = (*pd.add(pd_idx)) & 0x000F_FFFF_FFFF_F000;
        let pt = hhdm::phys_to_virt(pt_phys) as *mut u64;

        // Map the page with user flags
        *pt.add(pt_idx) = phys_frame | table_flags;
    }

    Ok(())
}

/// Traduz endereço virtual para físico no address space alvo
fn translate_in_target(target_cr3: u64, vaddr: u64) -> Option<u64> {
    unsafe {
        let pml4_phys = target_cr3 & !0xFFF;
        let pml4 = hhdm::phys_to_virt(pml4_phys) as *const u64;

        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pml4e = *pml4.add(pml4_idx);
        if pml4e & 1 == 0 {
            return None;
        }

        let pdpt_phys = pml4e & 0x000F_FFFF_FFFF_F000;
        let pdpt = hhdm::phys_to_virt(pdpt_phys) as *const u64;

        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pdpte = *pdpt.add(pdpt_idx);
        if pdpte & 1 == 0 {
            return None;
        }

        // 1GB page?
        if pdpte & 0x80 != 0 {
            let phys_base = pdpte & 0x000F_FFFF_C000_0000;
            return Some(phys_base | (vaddr & 0x3FFF_FFFF));
        }

        let pd_phys = pdpte & 0x000F_FFFF_FFFF_F000;
        let pd = hhdm::phys_to_virt(pd_phys) as *const u64;

        let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
        let pde = *pd.add(pd_idx);
        if pde & 1 == 0 {
            return None;
        }

        // 2MB page?
        if pde & 0x80 != 0 {
            let phys_base = pde & 0x000F_FFFF_FFE0_0000;
            return Some(phys_base | (vaddr & 0x1F_FFFF));
        }

        let pt_phys = pde & 0x000F_FFFF_FFFF_F000;
        let pt = hhdm::phys_to_virt(pt_phys) as *const u64;

        let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;
        let pte = *pt.add(pt_idx);
        if pte & 1 == 0 {
            return None;
        }

        let phys_base = pte & 0x000F_FFFF_FFFF_F000;
        Some(phys_base | (vaddr & 0xFFF))
    }
}
