//! # Funções de Alinhamento de Memória
//!
//! Utilitários para alinhamento de endereços e tamanhos.
//! Essencial para paginação e alocação de memória.
//!
//! ## Exemplos:
//! ```
//! use klib::primitives::align::{align_up, align_down, is_aligned};
//!
//! let page_size = 4096;
//! assert_eq!(align_up(5000, page_size), 8192);
//! assert_eq!(align_down(5000, page_size), 4096);
//! assert!(is_aligned(4096, page_size));
//! ```

/// Alinha valor para cima (próximo múltiplo de `align`).
///
/// ## Pré-condições:
/// - `align` deve ser potência de 2
///
/// ## Exemplos:
/// - `align_up(100, 64) = 128`
/// - `align_up(4096, 4096) = 4096`
/// - `align_up(0, 4096) = 0`
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

/// Alinha valor para baixo (múltiplo de `align` anterior ou igual).
///
/// ## Pré-condições:
/// - `align` deve ser potência de 2
///
/// ## Exemplos:
/// - `align_down(100, 64) = 64`
/// - `align_down(4096, 4096) = 4096`
/// - `align_down(0, 4096) = 0`
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}

/// Verifica se valor está alinhado.
///
/// ## Pré-condições:
/// - `align` deve ser potência de 2
#[inline]
pub const fn is_aligned(value: usize, align: usize) -> bool {
    debug_assert!(align.is_power_of_two());
    (value & (align - 1)) == 0
}

/// Calcula número de unidades de `align` necessárias para `size` bytes.
///
/// ## Exemplos:
/// - `div_round_up(100, 64) = 2` (precisa de 2 blocos de 64)
/// - `div_round_up(128, 64) = 2`
#[inline]
pub const fn div_round_up(size: usize, align: usize) -> usize {
    (size + align - 1) / align
}

/// Offset necessário para alinhar um ponteiro.
///
/// ## Retorno:
/// Número de bytes que precisam ser adicionados a `ptr` para ficar alinhado.
#[inline]
pub const fn align_offset(ptr: usize, align: usize) -> usize {
    let remainder = ptr % align;
    if remainder == 0 {
        0
    } else {
        align - remainder
    }
}
