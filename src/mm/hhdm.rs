//! # Higher Half Direct Map (HHDM)
//!
//! Mapeia toda a RAM física em uma região fixa do kernel space.
//!
//! O HHDM permite que o kernel acesse qualquer endereço físico simplesmente
//! adicionando HHDM_BASE ao endereço. Isso elimina a necessidade de identity
//! mapping na metade inferior e permite isolamento completo do userspace.
//!
//! ## Layout de Memória
//!
//! ```text
//! 0xFFFF_8000_0000_0000 ─┬─────────────────────────
//!                        │ HHDM (Direct Map RAM)
//!                        │ phys_to_virt(p) = HHDM + p
//!                        │ Toda RAM física mapeada
//! 0xFFFF_9000_0000_0000 ─┴─────────────────────────
//! ```
//!
//! ## Implementação no Boot
//!
//! - Bootloader (Ignite) mapeia toda RAM detectada em `HHDM_BASE + phys`
//! - Usa huge pages (2MB/1GB) para eficiência
//! - Global bit setado para não flush no context switch

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Base do Higher Half Direct Map
/// Toda RAM física é mapeada a partir deste endereço
pub const HHDM_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Fim da região HHDM (16TB de RAM máximo suportado)
pub const HHDM_END: u64 = 0xFFFF_9000_0000_0000;

/// Tamanho máximo de RAM suportada (16TB)
pub const HHDM_MAX_SIZE: u64 = HHDM_END - HHDM_BASE;

// =============================================================================
// STATE
// =============================================================================

/// Indica se o HHDM foi inicializado
static HHDM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Tamanho real da RAM física mapeada (em bytes)
static HHDM_MAPPED_SIZE: AtomicU64 = AtomicU64::new(0);

/// Offset aplicado pelo bootloader (se diferente de HHDM_BASE)
/// Usado para compatibilidade com bootloaders que usam offset diferente
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(HHDM_BASE);

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o HHDM com informações do bootloader
///
/// # Arguments
///
/// * `offset` - Offset usado pelo bootloader (normalmente HHDM_BASE)
/// * `size` - Tamanho da RAM física mapeada
///
/// # Safety
///
/// Deve ser chamado apenas uma vez durante early boot.
pub unsafe fn init(offset: u64, size: u64) {
    HHDM_OFFSET.store(offset, Ordering::Release);
    HHDM_MAPPED_SIZE.store(size, Ordering::Release);
    HHDM_INITIALIZED.store(true, Ordering::Release);
}

/// Verifica se o HHDM está inicializado
#[inline]
pub fn is_initialized() -> bool {
    HHDM_INITIALIZED.load(Ordering::Acquire)
}

/// Retorna o offset atual do HHDM
#[inline]
pub fn offset() -> u64 {
    HHDM_OFFSET.load(Ordering::Acquire)
}

/// Retorna o tamanho da RAM mapeada
#[inline]
pub fn mapped_size() -> u64 {
    HHDM_MAPPED_SIZE.load(Ordering::Acquire)
}

// =============================================================================
// ADDRESS CONVERSION
// =============================================================================

/// Converte endereço físico para virtual (HHDM)
///
/// # Arguments
///
/// * `phys` - Endereço físico
///
/// # Returns
///
/// Ponteiro para o endereço virtual correspondente
///
/// # Safety
///
/// O endereço físico deve estar dentro da RAM mapeada.
/// O tipo T deve ser compatível com o conteúdo naquele endereço.
#[inline(always)]
pub fn phys_to_virt<T>(phys: u64) -> *mut T {
    let offset = HHDM_OFFSET.load(Ordering::Relaxed);
    (offset + phys) as *mut T
}

/// Converte endereço físico para virtual, retornando referência
///
/// # Safety
///
/// - O endereço físico deve estar dentro da RAM mapeada
/// - O tipo T deve estar corretamente alinhado
/// - O tempo de vida retornado deve ser gerenciado pelo caller
#[inline(always)]
pub unsafe fn phys_to_ref<T>(phys: u64) -> &'static T {
    &*phys_to_virt::<T>(phys)
}

/// Converte endereço físico para virtual, retornando referência mutável
///
/// # Safety
///
/// - O endereço físico deve estar dentro da RAM mapeada
/// - O tipo T deve estar corretamente alinhado
/// - Não deve haver outras referências ativas
#[inline(always)]
pub unsafe fn phys_to_mut<T>(phys: u64) -> &'static mut T {
    &mut *phys_to_virt::<T>(phys)
}

/// Converte endereço virtual (HHDM) para físico
///
/// # Arguments
///
/// * `virt` - Endereço virtual na região HHDM
///
/// # Returns
///
/// Endereço físico correspondente
///
/// # Panics
///
/// Panics se o endereço não estiver na região HHDM (em debug builds)
#[inline(always)]
pub fn virt_to_phys(virt: u64) -> u64 {
    let offset = HHDM_OFFSET.load(Ordering::Relaxed);
    debug_assert!(
        virt >= offset,
        "virt_to_phys: address {:#x} is below HHDM base {:#x}",
        virt,
        offset
    );
    virt - offset
}

/// Verifica se um endereço virtual está na região HHDM
#[inline]
pub fn is_hhdm_address(virt: u64) -> bool {
    let offset = HHDM_OFFSET.load(Ordering::Relaxed);
    virt >= offset && virt < offset + HHDM_MAX_SIZE
}

/// Converte endereço virtual para físico de forma segura
///
/// # Returns
///
/// `Some(phys)` se o endereço está na região HHDM, `None` caso contrário
#[inline]
pub fn try_virt_to_phys(virt: u64) -> Option<u64> {
    if is_hhdm_address(virt) {
        Some(virt_to_phys(virt))
    } else {
        None
    }
}

// =============================================================================
// LEGACY COMPATIBILITY
// =============================================================================

/// Converte endereço físico para virtual usando offset legado
///
/// Usado durante transição do identity map para HHDM.
/// Funciona tanto com identity map (offset=0) quanto HHDM.
#[inline(always)]
pub fn phys_to_virt_legacy<T>(phys: u64) -> *mut T {
    // Durante early boot, pode não haver HHDM ainda
    if HHDM_INITIALIZED.load(Ordering::Acquire) {
        phys_to_virt(phys)
    } else {
        // Fallback para identity map
        phys as *mut T
    }
}

// =============================================================================
// UTILITIES
// =============================================================================

/// Zera uma página física
///
/// # Safety
///
/// O endereço físico deve ser válido e a página deve ser acessível
#[inline]
pub unsafe fn zero_page(phys: u64) {
    let ptr: *mut u8 = phys_to_virt(phys);
    core::ptr::write_bytes(ptr, 0, crate::mm::config::PAGE_SIZE);
}

/// Copia uma página física para outra
///
/// # Safety
///
/// Ambos os endereços físicos devem ser válidos e acessíveis
#[inline]
pub unsafe fn copy_page(src_phys: u64, dst_phys: u64) {
    let src: *const u8 = phys_to_virt(src_phys);
    let dst: *mut u8 = phys_to_virt(dst_phys);
    core::ptr::copy_nonoverlapping(src, dst, crate::mm::config::PAGE_SIZE);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_conversion() {
        unsafe { init(HHDM_BASE, 0x1_0000_0000) };

        let phys = 0x1000u64;
        let virt = phys_to_virt::<u8>(phys) as u64;
        assert_eq!(virt, HHDM_BASE + phys);

        let back = virt_to_phys(virt);
        assert_eq!(back, phys);
    }

    #[test]
    fn test_is_hhdm_address() {
        unsafe { init(HHDM_BASE, 0x1_0000_0000) };

        assert!(is_hhdm_address(HHDM_BASE));
        assert!(is_hhdm_address(HHDM_BASE + 0x1000));
        assert!(!is_hhdm_address(0x1000));
        assert!(!is_hhdm_address(HHDM_END + 0x1000));
    }
}
