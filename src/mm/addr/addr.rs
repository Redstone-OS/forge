//! Conversão Físico ↔ Virtual
//!
//! Fornece funções para converter endereços físicos em ponteiros virtuais
//! acessíveis e vice-versa. Crítico para operações de page table em higher-half.
//!
//! # Layout de Memória
//!
//! O kernel Redstone OS usa higher-half mapping:
//! - Kernel code: 0xFFFFFFFF80000000+ (PML4[511])
//! - Heap:        0xFFFF900000000000+ (PML4[288])
//! - Identity:    0x0000000000000000 - 0x0000000100000000 (0-4GB via huge pages)
//!
//! Para acessar memória física (page tables, bitmap PMM, etc), usamos
//! o identity map de 0-4GB que o bootloader cria.
//!
//! # Importante
//!
//! Este offset DEVE ser consistente com o layout do bootloader (Ignite).
//! Se o bootloader mudar o identity map, este valor deve ser ajustado.

use crate::mm::pmm::FRAME_SIZE;

/// Offset para conversão físico → virtual
///
/// O bootloader agora cria identity map para toda a RAM disponível, então:
/// - phys 0x1000 → virt 0x1000 (identity)
///
/// O limite é aumentado para 512GB para suportar sistemas com muita RAM.
/// Na prática, o bootloader mapeia apenas até o endereço físico máximo
/// reportado pelo memory map + margem.
pub const PHYS_IDENTITY_LIMIT: u64 = 0x80_0000_0000; // 512 GB

/// Converte endereço físico para ponteiro virtual acessível.
///
/// # Safety
///
/// - O endereço físico DEVE estar dentro do identity map (< 4GB)
/// - O caller DEVE garantir que a memória está mapeada e acessível
/// - O tipo T deve ter alinhamento compatível com o endereço
///
/// # Panics
///
/// Faz panic se `phys >= PHYS_IDENTITY_LIMIT` (não temos mapeamento)
#[inline(always)]
pub unsafe fn phys_to_virt<T>(phys: u64) -> *mut T {
    debug_assert!(
        phys < PHYS_IDENTITY_LIMIT,
        "(VMM) phys_to_virt: endereço {:#x} fora do identity map!",
        phys
    );

    // No identity map: virtual == physical para 0-4GB
    phys as *mut T
}

/// Converte ponteiro virtual (do identity map) de volta para endereço físico.
///
/// # Safety
///
/// O ponteiro DEVE estar dentro do identity map (< 4GB)
#[inline(always)]
pub fn virt_to_phys<T>(virt: *const T) -> u64 {
    let addr = virt as u64;
    debug_assert!(
        addr < PHYS_IDENTITY_LIMIT,
        "(VMM) virt_to_phys: endereço {:#x} fora do identity map!",
        addr
    );
    addr
}

/// Converte endereço físico para ponteiro virtual COM validação.
///
/// Retorna None se o endereço estiver fora do identity map.
#[inline]
pub fn try_phys_to_virt<T>(phys: u64) -> Option<*mut T> {
    if phys < PHYS_IDENTITY_LIMIT {
        Some(phys as *mut T)
    } else {
        None
    }
}

/// Valida que um endereço físico está dentro do identity map.
#[inline]
pub fn is_phys_accessible(phys: u64) -> bool {
    phys < PHYS_IDENTITY_LIMIT
}

/// Valida alinhamento de frame (4KB)
#[inline]
pub fn is_frame_aligned(addr: u64) -> bool {
    addr % FRAME_SIZE as u64 == 0
}

/// Alinha endereço para baixo ao limite de frame
#[inline]
pub fn frame_align_down(addr: u64) -> u64 {
    addr & !(FRAME_SIZE as u64 - 1)
}

/// Alinha endereço para cima ao limite de frame
#[inline]
pub fn frame_align_up(addr: u64) -> u64 {
    (addr + FRAME_SIZE as u64 - 1) & !(FRAME_SIZE as u64 - 1)
}
