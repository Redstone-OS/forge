//! # Funções de Alinhamento de Memória
//!
//! Funções utilitárias para alinhamento de endereços e valores.
//! Consolidadas da klib e mm/config para evitar duplicação.

/// Alinha um valor para cima ao próximo múltiplo de `align`.
///
/// # Exemplo
/// ```
/// assert_eq!(align_up(10, 4), 12);
/// assert_eq!(align_up(16, 4), 16);
/// ```
#[inline(always)]
pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

/// Alinha um valor para baixo ao múltiplo anterior de `align`.
///
/// # Exemplo
/// ```
/// assert_eq!(align_down(10, 4), 8);
/// assert_eq!(align_down(16, 4), 16);
/// ```
#[inline(always)]
pub const fn align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}

/// Verifica se um valor está alinhado a `align`.
///
/// # Exemplo
/// ```
/// assert!(is_aligned(16, 4));
/// assert!(!is_aligned(10, 4));
/// ```
#[inline(always)]
pub const fn is_aligned(val: usize, align: usize) -> bool {
    val & (align - 1) == 0
}

/// Versão de align_up para u64 (útil para endereços físicos).
#[inline(always)]
pub const fn align_up_u64(val: u64, align: u64) -> u64 {
    (val + align - 1) & !(align - 1)
}

/// Versão de align_down para u64.
#[inline(always)]
pub const fn align_down_u64(val: u64, align: u64) -> u64 {
    val & !(align - 1)
}

/// Versão de is_aligned para u64.
#[inline(always)]
pub const fn is_aligned_u64(val: u64, align: u64) -> bool {
    val & (align - 1) == 0
}
