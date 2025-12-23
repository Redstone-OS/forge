//! Virtual Memory Manager (VMM).
//!
//! Gerencia as Page Tables (PML4) e o mapeamento Virtual -> Físico.
//! Baseado na arquitetura x86_64 (4 níveis).

use crate::arch::platform::cpu::X64Cpu;
use crate::arch::traits::CpuOps;
use crate::mm::pmm::{PhysFrame, FRAME_ALLOCATOR, FRAME_SIZE};
use core::arch::asm;

// Flags de Paginação (x86_64)
pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_EXEC: u64 = 1 << 63;

/// Endereço da tabela PML4 ativa.
static mut ACTIVE_PML4_PHYS: u64 = 0;

/// Inicializa o VMM.
/// Assume que o bootloader já configurou uma paginação básica (identity map + kernel map).
pub unsafe fn init(boot_info: &crate::core::handoff::BootInfo) {
    // 1. Ler CR3 atual para saber onde está a PML4
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3);
    ACTIVE_PML4_PHYS = cr3 & 0x000F_FFFF_FFFF_F000; // Limpar flags do CR3 (PCID, PWT, etc)

    crate::kinfo!("VMM Initialized. PML4 at {:#x}", ACTIVE_PML4_PHYS);

    // Futuro: Criar uma nova PML4 limpa e trocar para ela para garantir
    // que não dependemos de mapeamentos sujos do bootloader.
}

/// Mapeia uma página virtual para um frame físico.
///
/// # Returns
/// `true` se sucesso, `false` se OOM (não conseguiu alocar tabelas intermediárias).
pub unsafe fn map_page(virt_addr: u64, phys_addr: u64, flags: u64) -> bool {
    let pml4_idx = (virt_addr >> 39) & 0x1FF;
    let pdpt_idx = (virt_addr >> 30) & 0x1FF;
    let pd_idx = (virt_addr >> 21) & 0x1FF;
    let pt_idx = (virt_addr >> 12) & 0x1FF;

    // Acesso à memória física das tabelas.
    // Como estamos em identity map (provavelmente) nas regiões baixas ou temos offset,
    // precisamos de uma função `phys_to_virt`.
    // Por enquanto, assumimos Identity Map para as tabelas de página (simplificação Fase 1).

    let pml4_ptr = ACTIVE_PML4_PHYS as *mut u64;
    let pml4_entry = &mut *pml4_ptr.add(pml4_idx as usize);

    let pdpt_phys = ensure_table_entry(pml4_entry);
    if pdpt_phys == 0 {
        return false;
    }

    let pdpt_ptr = pdpt_phys as *mut u64;
    let pdpt_entry = &mut *pdpt_ptr.add(pdpt_idx as usize);

    let pd_phys = ensure_table_entry(pdpt_entry);
    if pd_phys == 0 {
        return false;
    }

    let pd_ptr = pd_phys as *mut u64;
    let pd_entry = &mut *pd_ptr.add(pd_idx as usize);

    let pt_phys = ensure_table_entry(pd_entry);
    if pt_phys == 0 {
        return false;
    }

    let pt_ptr = pt_phys as *mut u64;
    let pt_entry = &mut *pt_ptr.add(pt_idx as usize);

    // Escrever a entrada final (Página)
    *pt_entry = (phys_addr & 0x000F_FFFF_FFFF_F000) | flags | PAGE_PRESENT;

    // Invalidar TLB
    asm!("invlpg [{}]", in(reg) virt_addr);

    true
}

/// Helper: Garante que uma entrada de tabela aponte para uma próxima tabela válida.
/// Se não existir, aloca um frame, zera e aponta.
/// Retorna o endereço físico da próxima tabela ou 0 se erro.
unsafe fn ensure_table_entry(entry: &mut u64) -> u64 {
    if *entry & PAGE_PRESENT != 0 {
        return *entry & 0x000F_FFFF_FFFF_F000;
    }

    // Alocar novo frame para a tabela
    let frame = FRAME_ALLOCATOR.lock().allocate_frame();
    if let Some(f) = frame {
        let phys = f.addr;
        // Zerar o frame recém alocado (IMPORTANTE! Lixo na tabela = crash)
        let ptr = phys as *mut u8;
        ptr.write_bytes(0, FRAME_SIZE);

        // Configurar entrada: Presente + Writable + User (permissão é refinada no nível final)
        *entry = phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        phys
    } else {
        0 // OOM
    }
}
