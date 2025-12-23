//! Kernel Library (KLib).
//!
//! Utilitários agnósticos de hardware para uso interno do Kernel.
//! Funciona como uma extensão da `core` library.

pub mod bitmap;
// Futuro: pub mod ring_buffer;
// Futuro: pub mod linked_list;

/// Alinha um endereço para cima.
///
/// # Exemplo
/// `align_up(10, 4) -> 12`
#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Alinha um endereço para baixo.
///
/// # Exemplo
/// `align_down(10, 4) -> 8`
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Verifica se um endereço está alinhado.
#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    (addr & (align - 1)) == 0
}
