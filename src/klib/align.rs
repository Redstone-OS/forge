//! Funções de alinhamento de memória

/// Alinha valor para cima
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

/// Alinha valor para baixo
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}

/// Verifica se está alinhado
#[inline]
pub const fn is_aligned(value: usize, align: usize) -> bool {
    debug_assert!(align.is_power_of_two());
    (value & (align - 1)) == 0
}
