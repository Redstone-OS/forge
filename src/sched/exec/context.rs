//! # Context Setup para Processos
//!
//! Configuração do contexto de CPU (trap frame) para primeira execução.
//! Este módulo abstrai os detalhes arquiteturais do x86_64.

use crate::arch::x86_64::interrupts::ExceptionStackFrame;
use crate::rmm::addr::VirtAddr;
use crate::rmm::virt::hhdm;

use super::config::{
    SWITCH_RESERVE, USER_CODE_SELECTOR, USER_DATA_SELECTOR, USER_RFLAGS, USER_STACK_TOP,
};
use super::error::ExecError;

/// Configura o trap frame inicial para um processo de usuário
///
/// Esta função escreve o ExceptionStackFrame na stack de kernel do processo
/// alvo, usando o HHDM para acesso seguro independente do CR3 ativo.
///
/// # Argumentos
///
/// * `target_cr3` - CR3 do Address Space do processo alvo
/// * `kstack_top` - Topo da stack de kernel do processo
/// * `entry_point` - Endereço de entrada do binário (e_entry do ELF)
///
/// # Retorna
///
/// * `Ok((rsp, rip))` - RSP e RIP iniciais para o contexto da task
/// * `Err(ExecError)` - Se não conseguir traduzir o endereço
///
/// # Safety
///
/// Esta função é unsafe porque:
/// - Escreve em memória via ponteiro raw
/// - Assume que kstack_top está mapeada em target_cr3
pub unsafe fn setup_user_trap_frame(
    target_cr3: u64,
    kstack_top: u64,
    entry_point: VirtAddr,
) -> Result<(u64, u64), ExecError> {
    // Traduzir endereço virtual da stack para físico
    let phys_top =
        translate_in_target(target_cr3, kstack_top - 8).ok_or(ExecError::MappingFailed)?;

    // Calcular endereço HHDM para escrita
    let phys_page = phys_top & !0xFFF;
    let offset = (kstack_top - 8) % crate::rmm::config::PAGE_SIZE as u64;

    let stack_top_hhdm = hhdm::phys_to_virt(phys_page) as *mut u8;
    let stack_top_ptr = stack_top_hhdm.add(offset as usize + 8);

    // Calcular posição do trap frame
    let frame_ptr = (stack_top_ptr as u64
        - core::mem::size_of::<ExceptionStackFrame>() as u64
        - SWITCH_RESERVE) as *mut ExceptionStackFrame;

    // Preencher trap frame
    (*frame_ptr).instruction_pointer = entry_point.as_u64();
    (*frame_ptr).code_segment = USER_CODE_SELECTOR;
    (*frame_ptr).cpu_flags = USER_RFLAGS;
    (*frame_ptr).stack_pointer = USER_STACK_TOP;
    (*frame_ptr).stack_segment = USER_DATA_SELECTOR;

    // Calcular RSP e RIP para o contexto da task
    let rsp = kstack_top - SWITCH_RESERVE;
    let rip = crate::sched::core::entry::user_entry_stub as u64;

    Ok((rsp, rip))
}

/// Traduz endereço virtual em endereço físico no Address Space alvo
///
/// Usa page table walk no CR3 especificado.
fn translate_in_target(target_cr3: u64, vaddr: u64) -> Option<u64> {
    // TODO: Usar RMM's AddressSpace::translate() quando disponível
    // Por agora, fazemos page table walk manual

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
