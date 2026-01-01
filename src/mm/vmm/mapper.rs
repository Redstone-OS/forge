#![allow(dead_code)]
//! Mapper: Funções de mapeamento de memória virtual
//!
//! Implementação real de mapeamento de páginas para x86_64.
//! Manipula tabelas de página hierárquicas (PML4 -> PDPT -> PD -> PT).

use super::vmm::MapFlags;
use core::arch::asm;

/// Constantes de paginação
const PAGE_SIZE: u64 = 4096;
const PAGE_MASK: u64 = 0x000F_FFFF_FFFF_F000;
const PT_ENTRIES: usize = 512;

/// Flags de entrada de página
const FLAG_PRESENT: u64 = 1 << 0;
const FLAG_WRITABLE: u64 = 1 << 1;
const FLAG_USER: u64 = 1 << 2;
const FLAG_NO_EXEC: u64 = 1 << 63;

/// Lê o registrador CR3 (endereço físico da PML4)
#[inline]
pub fn read_cr3() -> u64 {
    let cr3: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
    }
    cr3 & PAGE_MASK
}

/// Escreve no registrador CR3
#[inline]
pub unsafe fn write_cr3(value: u64) {
    asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

/// Cria uma nova PML4 (Page map Level 4) para um processo
/// Copia a parte do Kernel (Higher Half) da PML4 atual.
pub fn create_new_p4(pmm: &mut crate::mm::pmm::BitmapFrameAllocator) -> Result<u64, &'static str> {
    // Alocar frame para a nova PML4
    let frame = pmm.allocate_frame().ok_or("(Mapper) OOM allocating PML4")?;
    let pml4_phys = frame.as_u64();

    // Zerar (IMPORTANTE para garantir que USER space esteja vazio)
    unsafe {
        zero_page(pml4_phys);
    }

    // Copiar kernel mappings (Entradas 256 a 511) da P4 ATUAL
    // Isso garante que herdamos stacks de tasks já criadas e expansões do heap.
    let current_pml4 = read_cr3();

    // Acessar via HHDM (mais seguro que identity)
    unsafe {
        let src_ptr: *const u64 = crate::mm::addr::phys_to_virt(current_pml4);
        let dst_ptr: *mut u64 = crate::mm::addr::phys_to_virt(pml4_phys);

        // Copiar metade superior (Kernel)
        for i in 256..512 {
            let entry = core::ptr::read_volatile(src_ptr.add(i));
            core::ptr::write_volatile(dst_ptr.add(i), entry);
        }

        // Entradas 0..255 (User Space) permanecem ZERADAS (via zero_page no início desta função).
        // O ELF loader e o gerenciador de VMAs serão responsáveis por povoar esta área de forma isolada.
    }

    Ok(pml4_phys)
}

/// Concede acesso de usuário para um endereço virtual existente (Atualiza flags)
pub fn grant_user_access(page_virt: u64) {
    let pml4_phys = read_cr3();

    let pml4_idx = ((page_virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((page_virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((page_virt >> 12) & 0x1FF) as usize;

    unsafe {
        // PML4
        let mut pml4e = get_table_entry(pml4_phys, pml4_idx);
        if pml4e & FLAG_PRESENT != 0 {
            pml4e |= FLAG_USER;
            set_table_entry(pml4_phys, pml4_idx, pml4e);
        } else {
            return;
        }

        let pdpt_phys = pml4e & PAGE_MASK;

        // PDPT
        let mut pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        if pdpte & FLAG_PRESENT != 0 {
            pdpte |= FLAG_USER;
            set_table_entry(pdpt_phys, pdpt_idx, pdpte);

            // Huge Page Check (1GB)
            if pdpte & (1 << 7) != 0 {
                return;
            }
        } else {
            return;
        }

        let pd_phys = pdpte & PAGE_MASK;

        // PD
        let mut pde = get_table_entry(pd_phys, pd_idx);
        if pde & FLAG_PRESENT != 0 {
            pde |= FLAG_USER;
            set_table_entry(pd_phys, pd_idx, pde);

            // Huge Page Check (2MB)
            if pde & (1 << 7) != 0 {
                return;
            }
        } else {
            return;
        }

        let pt_phys = pde & PAGE_MASK;

        // PT
        let mut pte = get_table_entry(pt_phys, pt_idx);
        if pte & FLAG_PRESENT != 0 {
            pte |= FLAG_USER;
            set_table_entry(pt_phys, pt_idx, pte);
        }

        // Flush TLB
        asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
    }
}

/// Obtém entrada de uma tabela de página
/// Usa identity mapping para acessar tabelas de página
#[inline]
unsafe fn get_table_entry(table_phys: u64, index: usize) -> u64 {
    let table_ptr: *const u64 = crate::mm::addr::phys_to_virt(table_phys);
    core::ptr::read_volatile(table_ptr.add(index))
}

/// Escreve entrada em uma tabela de página
/// Usa identity mapping para acessar tabelas de página
#[inline]
unsafe fn set_table_entry(table_phys: u64, index: usize, value: u64) {
    let table_ptr: *mut u64 = crate::mm::addr::phys_to_virt(table_phys);
    core::ptr::write_volatile(table_ptr.add(index), value);
}

/// Traduz endereço virtual para físico usando uma PML4 específica
pub fn translate_addr_in_p4(pml4_phys: u64, virt: u64) -> Option<u64> {
    let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt >> 12) & 0x1FF) as usize;
    let offset = virt & 0xFFF;

    unsafe {
        // PML4 -> PDPT
        let pml4e = get_table_entry(pml4_phys, pml4_idx);
        if pml4e & FLAG_PRESENT == 0 {
            return None;
        }
        let pdpt_phys = pml4e & PAGE_MASK;

        // PDPT -> PD
        let pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        if pdpte & FLAG_PRESENT == 0 {
            return None;
        }
        if pdpte & (1 << 7) != 0 {
            return Some((pdpte & 0x000F_FFFF_C000_0000) | (virt & 0x3FFF_FFFF));
        }
        let pd_phys = pdpte & PAGE_MASK;

        // PD -> PT
        let pde = get_table_entry(pd_phys, pd_idx);
        if pde & FLAG_PRESENT == 0 {
            return None;
        }
        if pde & (1 << 7) != 0 {
            return Some((pde & 0x000F_FFFF_FFE0_0000) | (virt & 0x1F_FFFF));
        }
        let pt_phys = pde & PAGE_MASK;

        // PT -> Frame
        let pte = get_table_entry(pt_phys, pt_idx);
        if pte & FLAG_PRESENT == 0 {
            return None;
        }
        let frame_phys = pte & PAGE_MASK;

        Some(frame_phys | offset)
    }
}

/// Traduz endereço virtual para físico usando as tabelas de página atuais
pub fn translate_addr(virt: u64) -> Option<u64> {
    translate_addr_in_p4(read_cr3(), virt)
}

/// Mapeia uma página virtual para um frame físico
///
/// NOTA: Assume que todas as tabelas intermediárias (PDPT, PD, PT) já existem.
/// Se uma tabela intermediária não existir, retorna erro.
/// Para criar tabelas automaticamente, use `map_page_with_pmm`.
pub fn map_page(page_virt: u64, frame_phys: u64, flags: MapFlags) -> Result<(), &'static str> {
    let pml4_phys = read_cr3();

    let pml4_idx = ((page_virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((page_virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((page_virt >> 12) & 0x1FF) as usize;

    // Converte MapFlags para flags de PTE
    let mut pte_flags = FLAG_PRESENT;
    if flags.contains(MapFlags::WRITABLE) {
        pte_flags |= FLAG_WRITABLE;
    }
    if flags.contains(MapFlags::USER) {
        pte_flags |= FLAG_USER;
    }
    if !flags.contains(MapFlags::EXECUTABLE) || flags.contains(MapFlags::NO_EXECUTE) {
        pte_flags |= FLAG_NO_EXEC;
    }

    unsafe {
        // PML4 -> PDPT
        let pml4e = get_table_entry(pml4_phys, pml4_idx);
        if pml4e & FLAG_PRESENT == 0 {
            return Err("(VMM) PML4E não presente");
        }
        let pdpt_phys = pml4e & PAGE_MASK;

        // PDPT -> PD
        let pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        if pdpte & FLAG_PRESENT == 0 {
            return Err("(VMM) PDPTE não presente");
        }
        let pd_phys = pdpte & PAGE_MASK;

        // PD -> PT
        let pde = get_table_entry(pd_phys, pd_idx);
        if pde & FLAG_PRESENT == 0 {
            return Err("(VMM) PDE não presente");
        }
        let pt_phys = pde & PAGE_MASK;

        // Escreve a PTE final
        let pte = frame_phys | pte_flags;
        set_table_entry(pt_phys, pt_idx, pte);

        // Invalida TLB para esta página
        asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
    }

    Ok(())
}

/// Desmapeia uma página virtual
pub fn unmap_page(page_virt: u64) -> Result<(), &'static str> {
    let pml4_phys = read_cr3();

    let pml4_idx = ((page_virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((page_virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((page_virt >> 12) & 0x1FF) as usize;

    unsafe {
        // Navega até a PT
        let pml4e = get_table_entry(pml4_phys, pml4_idx);
        if pml4e & FLAG_PRESENT == 0 {
            return Ok(()); // Já não está mapeada
        }
        let pdpt_phys = pml4e & PAGE_MASK;

        let pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        if pdpte & FLAG_PRESENT == 0 {
            return Ok(());
        }
        let pd_phys = pdpte & PAGE_MASK;

        let pde = get_table_entry(pd_phys, pd_idx);
        if pde & FLAG_PRESENT == 0 {
            return Ok(());
        }
        let pt_phys = pde & PAGE_MASK;

        // Limpa a PTE
        set_table_entry(pt_phys, pt_idx, 0);

        // Invalida TLB
        asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
    }

    Ok(())
}

/// Mapeia página, criando tabelas intermediárias se necessário
///
/// Usa o PMM para alocar frames para novas tabelas de página.
pub fn map_page_with_pmm(
    page_virt: u64,
    frame_phys: u64,
    flags: MapFlags,
    pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
) -> Result<(), &'static str> {
    let pml4_phys = read_cr3();

    let pml4_idx = ((page_virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((page_virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((page_virt >> 12) & 0x1FF) as usize;

    // Converte MapFlags para flags de PTE do hardware
    let mut pte_flags = FLAG_PRESENT;
    if flags.contains(MapFlags::WRITABLE) {
        pte_flags |= FLAG_WRITABLE;
    }
    if flags.contains(MapFlags::USER) {
        pte_flags |= FLAG_USER;
    }
    if !flags.contains(MapFlags::EXECUTABLE) || flags.contains(MapFlags::NO_EXECUTE) {
        pte_flags |= FLAG_NO_EXEC;
    }

    // Flags para tabelas intermediárias (sempre presentes e graváveis)
    let mut table_flags = FLAG_PRESENT | FLAG_WRITABLE;
    if flags.contains(MapFlags::USER) {
        table_flags |= FLAG_USER;
    }

    unsafe {
        // Garante que PDPT existe
        let mut pml4e = get_table_entry(pml4_phys, pml4_idx);
        let pdpt_phys: u64;
        if pml4e & FLAG_PRESENT == 0 {
            // Aloca nova PDPT
            let new_pdpt = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PDPT")?;
            pdpt_phys = new_pdpt.addr();
            // Zera a nova tabela
            zero_page(pdpt_phys);
            // Atualiza PML4E
            pml4e = pdpt_phys | table_flags;
            set_table_entry(pml4_phys, pml4_idx, pml4e);
        } else {
            // Se já existe, atualizar flags (USER e WRITABLE)
            let mut changed = false;
            if flags.contains(MapFlags::USER) && (pml4e & FLAG_USER == 0) {
                pml4e |= FLAG_USER;
                changed = true;
            }
            if flags.contains(MapFlags::WRITABLE) && (pml4e & FLAG_WRITABLE == 0) {
                pml4e |= FLAG_WRITABLE;
                changed = true;
            }
            if changed {
                set_table_entry(pml4_phys, pml4_idx, pml4e);
            }
            pdpt_phys = pml4e & PAGE_MASK;
        }

        // Garante que PD existe
        let mut pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        let pd_phys: u64;
        if pdpte & FLAG_PRESENT == 0 {
            // Aloca nova PD
            let new_pd = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PD")?;
            pd_phys = new_pd.addr();
            zero_page(pd_phys);
            pdpte = pd_phys | table_flags;
            set_table_entry(pdpt_phys, pdpt_idx, pdpte);
        } else {
            // Se já existe, atualizar flags (USER e WRITABLE)
            let mut changed = false;
            if flags.contains(MapFlags::USER) && (pdpte & FLAG_USER == 0) {
                pdpte |= FLAG_USER;
                changed = true;
            }
            if flags.contains(MapFlags::WRITABLE) && (pdpte & FLAG_WRITABLE == 0) {
                pdpte |= FLAG_WRITABLE;
                changed = true;
            }
            if changed {
                set_table_entry(pdpt_phys, pdpt_idx, pdpte);
            }
            pd_phys = pdpte & PAGE_MASK;
        }

        // Garante que PT existe
        let mut pde = get_table_entry(pd_phys, pd_idx);
        let pt_phys: u64;
        if pde & FLAG_PRESENT == 0 {
            // Aloca nova PT
            let new_pt = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PT")?;
            pt_phys = new_pt.addr();
            zero_page(pt_phys);
            pde = pt_phys | table_flags;
            set_table_entry(pd_phys, pd_idx, pde);
        } else {
            // Se já existe, atualizar flags (USER e WRITABLE)
            let mut changed = false;
            if flags.contains(MapFlags::USER) && (pde & FLAG_USER == 0) {
                pde |= FLAG_USER;
                changed = true;
            }
            if flags.contains(MapFlags::WRITABLE) && (pde & FLAG_WRITABLE == 0) {
                pde |= FLAG_WRITABLE;
                changed = true;
            }
            if changed {
                set_table_entry(pd_phys, pd_idx, pde);
            }
            pt_phys = pde & PAGE_MASK;
        }

        // Finalmente, escreve a PTE
        let pte = frame_phys | pte_flags;
        set_table_entry(pt_phys, pt_idx, pte);

        // Invalida TLB
        asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
    }

    Ok(())
}

/// Mapeia página em uma P4 específica (não necessariamente a atual)
///
/// Útil para mapear kernel stack no P4 de um processo recém-criado.
/// Usa identity mapping para acessar as tabelas de página.
pub fn map_page_in_target_p4(
    target_p4: u64,
    page_virt: u64,
    frame_phys: u64,
    flags: MapFlags,
    pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
) -> Result<(), &'static str> {
    let pml4_idx = ((page_virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((page_virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((page_virt >> 12) & 0x1FF) as usize;

    // Converte MapFlags para flags de PTE
    let mut pte_flags = FLAG_PRESENT;
    if flags.contains(MapFlags::WRITABLE) {
        pte_flags |= FLAG_WRITABLE;
    }
    if flags.contains(MapFlags::USER) {
        pte_flags |= FLAG_USER;
    }
    if !flags.contains(MapFlags::EXECUTABLE) || flags.contains(MapFlags::NO_EXECUTE) {
        pte_flags |= FLAG_NO_EXEC;
    }

    // Flags para tabelas intermediárias
    let mut table_flags = FLAG_PRESENT | FLAG_WRITABLE;
    if flags.contains(MapFlags::USER) {
        table_flags |= FLAG_USER;
    }

    unsafe {
        // Garante que PDPT existe na target P4
        let mut pml4e = get_table_entry(target_p4, pml4_idx);
        let pdpt_phys: u64;
        if pml4e & FLAG_PRESENT == 0 {
            let new_pdpt = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PDPT")?;
            pdpt_phys = new_pdpt.addr();
            zero_page(pdpt_phys);
            pml4e = pdpt_phys | table_flags;
            set_table_entry(target_p4, pml4_idx, pml4e);
        } else {
            if flags.contains(MapFlags::USER) && (pml4e & FLAG_USER == 0) {
                pml4e |= FLAG_USER;
                set_table_entry(target_p4, pml4_idx, pml4e);
            }
            pdpt_phys = pml4e & PAGE_MASK;
        }

        // Garante que PD existe
        let mut pdpte = get_table_entry(pdpt_phys, pdpt_idx);
        let pd_phys: u64;
        if pdpte & FLAG_PRESENT == 0 {
            let new_pd = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PD")?;
            pd_phys = new_pd.addr();
            zero_page(pd_phys);
            pdpte = pd_phys | table_flags;
            set_table_entry(pdpt_phys, pdpt_idx, pdpte);
        } else {
            if flags.contains(MapFlags::USER) && (pdpte & FLAG_USER == 0) {
                pdpte |= FLAG_USER;
                set_table_entry(pdpt_phys, pdpt_idx, pdpte);
            }
            pd_phys = pdpte & PAGE_MASK;
        }

        // Garante que PT existe
        let mut pde = get_table_entry(pd_phys, pd_idx);
        let pt_phys: u64;
        if pde & FLAG_PRESENT == 0 {
            let new_pt = pmm.allocate_frame().ok_or("(VMM) OOM ao alocar PT")?;
            pt_phys = new_pt.addr();
            zero_page(pt_phys);
            pde = pt_phys | table_flags;
            set_table_entry(pd_phys, pd_idx, pde);
        } else {
            if flags.contains(MapFlags::USER) && (pde & FLAG_USER == 0) {
                pde |= FLAG_USER;
                set_table_entry(pd_phys, pd_idx, pde);
            }
            pt_phys = pde & PAGE_MASK;
        }

        // Escreve a PTE final
        let pte = frame_phys | pte_flags;
        set_table_entry(pt_phys, pt_idx, pte);

        // Não faz invlpg aqui pois a target P4 pode não estar ativa
    }

    Ok(())
}

/// Zera uma página física (usada para novas tabelas de página)
#[inline]
unsafe fn zero_page(phys_addr: u64) {
    let ptr: *mut u64 = crate::mm::addr::phys_to_virt(phys_addr);
    let mut i = 0;
    while i < (PAGE_SIZE as usize / 8) {
        core::ptr::write_volatile(ptr.add(i), 0);
        i += 1;
    }
}
