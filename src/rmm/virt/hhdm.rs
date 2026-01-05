//! # Higher Half Direct Map (HHDM)
//!
//! Mapeamento direto de toda RAM física no higher half do espaço virtual.
//!
//! ## Funcionamento
//!
//! O bootloader (Limine) configura um mapeamento linear de toda a RAM física:
//!
//! ```text
//! Virtual = HHDM_BASE + Physical
//! Physical = Virtual - HHDM_BASE
//! ```
//!
//! ## Uso
//!
//! O HHDM permite que o kernel acesse qualquer endereço físico diretamente
//! através de aritmética simples, sem precisar criar mapeamentos temporários.
//!
//! ```rust
//! // Acessar frame físico
//! let phys = 0x1000;
//! let virt = hhdm::phys_to_virt(phys);
//! let ptr = virt as *mut u8;
//! unsafe { *ptr = 42; }
//! ```
//!
//! ## Layout
//!
//! ```text
//! 0xFFFF_8000_0000_0000 + 0x0000_0000 = RAM @ 0 MB
//! 0xFFFF_8000_0000_0000 + 0x0100_0000 = RAM @ 16 MB
//! 0xFFFF_8000_0000_0000 + 0x1_0000_0000 = RAM @ 4 GB
//! ...
//! ```

use crate::core::boot::BootInfo;
use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::{HHDM_BASE, HHDM_SIZE, PAGE_SIZE};

/// Offset do HHDM (configurado pelo bootloader)
static mut HHDM_OFFSET: u64 = HHDM_BASE;

/// Flag indicando se HHDM está inicializado
static mut HHDM_INITIALIZED: bool = false;

/// Tamanho da RAM física mapeada
static mut RAM_SIZE: u64 = 0;

/// Inicializa HHDM com offset do bootloader
///
/// # Safety
///
/// Deve ser chamada apenas uma vez durante o boot.
pub unsafe fn init(boot_info: &'static BootInfo) {
    // O bootloader (Ignite) configura o HHDM offset
    // Se for 0, usa o padrão HHDM_BASE
    HHDM_OFFSET = if boot_info.hhdm_offset != 0 {
        boot_info.hhdm_offset
    } else {
        HHDM_BASE
    };
    RAM_SIZE = boot_info.hhdm_size;
    HHDM_INITIALIZED = true;

    crate::kinfo!(
        "(RMM/HHDM) Inicializado: base=0x{:016x}, RAM={} MB",
        HHDM_OFFSET,
        RAM_SIZE / (1024 * 1024)
    );
}

/// Retorna o offset do HHDM
#[inline]
pub fn offset() -> u64 {
    unsafe { HHDM_OFFSET }
}

/// Verifica se HHDM está inicializado
#[inline]
pub fn is_initialized() -> bool {
    unsafe { HHDM_INITIALIZED }
}

/// Retorna tamanho da RAM física
#[inline]
pub fn ram_size() -> u64 {
    unsafe { RAM_SIZE }
}

// =============================================================================
// Conversões
// =============================================================================

/// Converte endereço físico para virtual (HHDM)
///
/// # Arguments
///
/// * `phys` - Endereço físico
///
/// # Returns
///
/// Endereço virtual correspondente no HHDM
///
/// # Panics
///
/// Em debug builds, panic se HHDM não inicializado.
#[inline]
pub fn phys_to_virt(phys: u64) -> u64 {
    debug_assert!(unsafe { HHDM_INITIALIZED }, "HHDM not initialized");
    unsafe { HHDM_OFFSET + phys }
}

/// Converte endereço físico para ponteiro tipado
///
/// # Safety
///
/// O caller deve garantir que:
/// - O endereço físico é válido
/// - O tipo T é apropriado para os dados no endereço
/// - Alinhamento está correto
#[inline]
pub fn phys_to_virt_ptr<T>(phys: u64) -> *mut T {
    phys_to_virt(phys) as *mut T
}

/// Converte PhysAddr para VirtAddr
#[inline]
pub fn phys_to_virt_addr(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys_to_virt(phys.as_u64()))
}

/// Converte endereço virtual HHDM para físico
///
/// # Arguments
///
/// * `virt` - Endereço virtual no range HHDM
///
/// # Returns
///
/// Some(phys) se o endereço está no HHDM, None caso contrário.
#[inline]
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    let offset = unsafe { HHDM_OFFSET };
    if virt >= offset && virt < offset + HHDM_SIZE {
        Some(virt - offset)
    } else {
        None
    }
}

/// Converte ponteiro para endereço físico
#[inline]
pub fn ptr_to_phys<T>(ptr: *const T) -> Option<u64> {
    virt_to_phys(ptr as u64)
}

/// Converte VirtAddr para PhysAddr (se for HHDM)
#[inline]
pub fn virt_to_phys_addr(virt: VirtAddr) -> Option<PhysAddr> {
    virt_to_phys(virt.as_u64()).map(PhysAddr::new)
}

// =============================================================================
// Checks
// =============================================================================

/// Verifica se endereço virtual está no HHDM
#[inline]
pub fn is_hhdm(virt: u64) -> bool {
    let offset = unsafe { HHDM_OFFSET };
    virt >= offset && virt < offset + HHDM_SIZE
}

/// Verifica se VirtAddr está no HHDM
#[inline]
pub fn is_hhdm_addr(virt: VirtAddr) -> bool {
    is_hhdm(virt.as_u64())
}

/// Verifica se endereço físico está dentro da RAM
#[inline]
pub fn is_valid_phys(phys: u64) -> bool {
    phys < unsafe { RAM_SIZE }
}

// =============================================================================
// Operações em Páginas
// =============================================================================

/// Zera uma página física via HHDM
///
/// # Safety
///
/// O caller deve garantir que:
/// - O endereço físico é válido e alinhado
/// - A página pode ser escrita
pub unsafe fn zero_page(phys: PhysAddr) {
    debug_assert!(phys.is_page_aligned(), "phys not page aligned");
    let ptr = phys_to_virt_ptr::<u8>(phys.as_u64());
    core::ptr::write_bytes(ptr, 0, PAGE_SIZE);
}

/// Copia uma página física para outra via HHDM
///
/// # Safety
///
/// O caller deve garantir que:
/// - Os endereços físicos são válidos e alinhados
/// - Não há overlap
pub unsafe fn copy_page(src: PhysAddr, dst: PhysAddr) {
    debug_assert!(src.is_page_aligned(), "src not page aligned");
    debug_assert!(dst.is_page_aligned(), "dst not page aligned");

    let src_ptr = phys_to_virt_ptr::<u8>(src.as_u64());
    let dst_ptr = phys_to_virt_ptr::<u8>(dst.as_u64());
    core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, PAGE_SIZE);
}

/// Compara duas páginas físicas
///
/// Retorna true se idênticas.
pub unsafe fn compare_pages(a: PhysAddr, b: PhysAddr) -> bool {
    let a_ptr = phys_to_virt_ptr::<u8>(a.as_u64());
    let b_ptr = phys_to_virt_ptr::<u8>(b.as_u64());

    for i in 0..PAGE_SIZE {
        if *a_ptr.add(i) != *b_ptr.add(i) {
            return false;
        }
    }
    true
}

/// Preenche página física com valor
pub unsafe fn fill_page(phys: PhysAddr, value: u8) {
    debug_assert!(phys.is_page_aligned(), "phys not page aligned");
    let ptr = phys_to_virt_ptr::<u8>(phys.as_u64());
    core::ptr::write_bytes(ptr, value, PAGE_SIZE);
}

/// Lê bytes de endereço físico
pub unsafe fn read_phys<T: Copy>(phys: u64) -> T {
    let ptr = phys_to_virt_ptr::<T>(phys);
    core::ptr::read_volatile(ptr)
}

/// Escreve bytes em endereço físico
pub unsafe fn write_phys<T>(phys: u64, value: T) {
    let ptr = phys_to_virt_ptr::<T>(phys);
    core::ptr::write_volatile(ptr, value);
}

/// Lê slice de bytes de endereço físico
pub unsafe fn read_phys_slice(phys: u64, buf: &mut [u8]) {
    let src = phys_to_virt_ptr::<u8>(phys);
    core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), buf.len());
}

/// Escreve slice de bytes em endereço físico
pub unsafe fn write_phys_slice(phys: u64, buf: &[u8]) {
    let dst = phys_to_virt_ptr::<u8>(phys);
    core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, buf.len());
}

// =============================================================================
// Referências a Estruturas Físicas
// =============================================================================

/// Obtém referência a estrutura em endereço físico
///
/// # Safety
///
/// - Endereço deve ser válido e alinhado
/// - Lifetime da referência deve ser menor que a vida da RAM
pub unsafe fn phys_ref<T>(phys: PhysAddr) -> &'static T {
    &*(phys_to_virt_ptr::<T>(phys.as_u64()))
}

/// Obtém referência mutável a estrutura em endereço físico
///
/// # Safety
///
/// - Endereço deve ser válido e alinhado
/// - Acesso exclusivo deve ser garantido externamente
pub unsafe fn phys_ref_mut<T>(phys: PhysAddr) -> &'static mut T {
    &mut *(phys_to_virt_ptr::<T>(phys.as_u64()))
}

/// Obtém slice de estruturas em range físico
pub unsafe fn phys_slice<T>(phys: PhysAddr, count: usize) -> &'static [T] {
    core::slice::from_raw_parts(phys_to_virt_ptr::<T>(phys.as_u64()), count)
}

/// Obtém slice mutável de estruturas em range físico
pub unsafe fn phys_slice_mut<T>(phys: PhysAddr, count: usize) -> &'static mut [T] {
    core::slice::from_raw_parts_mut(phys_to_virt_ptr::<T>(phys.as_u64()), count)
}
