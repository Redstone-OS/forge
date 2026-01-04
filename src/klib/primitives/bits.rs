//! # Operações de Manipulação de Bits
//!
//! Funções otimizadas para manipulação de bits usando instruções nativas.
//! Essencial para bitmap, flags e operações de baixo nível.

/// Encontra índice do primeiro bit 1 (Bit Scan Forward).
///
/// Equivalente à instrução `bsf` em x86.
///
/// ## Retorno:
/// - `Some(index)` se encontrou bit 1
/// - `None` se valor é 0
#[inline]
pub const fn bsf(value: u64) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(value.trailing_zeros())
    }
}

/// Encontra índice do último bit 1 (Bit Scan Reverse).
///
/// Equivalente à instrução `bsr` em x86.
///
/// ## Retorno:
/// - `Some(index)` se encontrou bit 1
/// - `None` se valor é 0
#[inline]
pub const fn bsr(value: u64) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(63 - value.leading_zeros())
    }
}

/// Conta número de bits 1 (Population Count).
///
/// Equivalente à instrução `popcnt` em x86.
#[inline]
pub const fn popcount(value: u64) -> u32 {
    value.count_ones()
}

/// Verifica se valor é potência de 2.
///
/// Nota: 0 não é considerado potência de 2.
#[inline]
pub const fn is_power_of_two(value: usize) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

/// Próxima potência de 2 maior ou igual ao valor.
///
/// ## Exemplos:
/// - `next_power_of_two(5) = 8`
/// - `next_power_of_two(8) = 8`
/// - `next_power_of_two(0) = 1`
#[inline]
pub const fn next_power_of_two(value: usize) -> usize {
    if value == 0 {
        return 1;
    }
    if is_power_of_two(value) {
        return value;
    }
    1 << (usize::BITS - (value - 1).leading_zeros())
}

/// Extrai um campo de bits de um valor.
///
/// ## Parâmetros:
/// - `value`: Valor fonte
/// - `start`: Bit inicial (0-indexed)
/// - `len`: Número de bits a extrair
#[inline]
pub const fn extract_bits(value: u64, start: u32, len: u32) -> u64 {
    let mask = (1u64 << len) - 1;
    (value >> start) & mask
}

/// Insere um campo de bits em um valor.
///
/// ## Parâmetros:
/// - `target`: Valor destino
/// - `value`: Valor a inserir
/// - `start`: Bit inicial (0-indexed)
/// - `len`: Número de bits
#[inline]
pub const fn insert_bits(target: u64, value: u64, start: u32, len: u32) -> u64 {
    let mask = (1u64 << len) - 1;
    let clear_mask = !(mask << start);
    (target & clear_mask) | ((value & mask) << start)
}

/// Testa se um bit específico está setado.
#[inline]
pub const fn test_bit(value: u64, bit: u32) -> bool {
    (value & (1u64 << bit)) != 0
}

/// Seta um bit específico.
#[inline]
pub const fn set_bit(value: u64, bit: u32) -> u64 {
    value | (1u64 << bit)
}

/// Limpa um bit específico.
#[inline]
pub const fn clear_bit(value: u64, bit: u32) -> u64 {
    value & !(1u64 << bit)
}

/// Toggle um bit específico.
#[inline]
pub const fn toggle_bit(value: u64, bit: u32) -> u64 {
    value ^ (1u64 << bit)
}

/// Rotaciona bits para a esquerda.
#[inline]
pub const fn rotate_left(value: u64, count: u32) -> u64 {
    value.rotate_left(count)
}

/// Rotaciona bits para a direita.
#[inline]
pub const fn rotate_right(value: u64, count: u32) -> u64 {
    value.rotate_right(count)
}

/// Conta zeros à esquerda (leading zeros).
#[inline]
pub const fn clz(value: u64) -> u32 {
    value.leading_zeros()
}

/// Conta zeros à direita (trailing zeros).
#[inline]
pub const fn ctz(value: u64) -> u32 {
    value.trailing_zeros()
}
