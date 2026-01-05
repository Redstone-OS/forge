//! # Bitmap Utilities
//!
//! Funções auxiliares para manipulação de bitmaps de frames.
//!
//! ## Operações
//!
//! - `find_first_zero`: Encontra primeiro bit livre
//! - `find_contiguous_zeros`: Encontra N bits livres consecutivos
//! - `set_range`: Marca range de bits como usados
//! - `clear_range`: Marca range de bits como livres
//! - `count_zeros`: Conta bits livres
//! - `count_ones`: Conta bits usados

/// Encontra primeiro bit zero em uma word
///
/// Retorna índice do bit (0-63) ou None se todos estão 1.
#[inline]
pub fn find_first_zero(bitmap: u64) -> Option<usize> {
    if bitmap == u64::MAX {
        return None;
    }
    Some((!bitmap).trailing_zeros() as usize)
}

/// Encontra primeiro bit um em uma word
///
/// Retorna índice do bit (0-63) ou None se todos estão 0.
#[inline]
pub fn find_first_one(bitmap: u64) -> Option<usize> {
    if bitmap == 0 {
        return None;
    }
    Some(bitmap.trailing_zeros() as usize)
}

/// Conta bits zero em uma word
#[inline]
pub fn count_zeros(bitmap: u64) -> usize {
    bitmap.count_zeros() as usize
}

/// Conta bits um em uma word
#[inline]
pub fn count_ones(bitmap: u64) -> usize {
    bitmap.count_ones() as usize
}

/// Encontra N bits zero contíguos em um array de words
///
/// Retorna índice global do primeiro bit do run, ou None se não encontrado.
///
/// # Argumentos
///
/// * `bitmap` - Array de u64 representando o bitmap
/// * `count` - Número de bits contíguos necessários
///
/// # Exemplo
///
/// ```
/// let bitmap = [0x0000_FFFF_FFFF_FFFF_u64, 0]; // 16 zeros no início da word 0
/// let start = find_contiguous_zeros(&bitmap, 8).unwrap();
/// assert_eq!(start, 48); // bits 48-55 estão livres
/// ```
pub fn find_contiguous_zeros(bitmap: &[u64], count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    let total_bits = bitmap.len() * 64;
    let mut run_start = 0;
    let mut run_len = 0;

    for global_bit in 0..total_bits {
        let word_idx = global_bit / 64;
        let bit_idx = global_bit % 64;

        let is_zero = (bitmap[word_idx] & (1 << bit_idx)) == 0;

        if is_zero {
            if run_len == 0 {
                run_start = global_bit;
            }
            run_len += 1;
            if run_len >= count {
                return Some(run_start);
            }
        } else {
            run_len = 0;
        }
    }

    None
}

/// Encontra N bits zero contíguos alinhados
///
/// Similar a `find_contiguous_zeros` mas garante que o início
/// está alinhado a `alignment` bits.
pub fn find_aligned_zeros(bitmap: &[u64], count: usize, alignment: usize) -> Option<usize> {
    if count == 0 || alignment == 0 {
        return None;
    }

    let total_bits = bitmap.len() * 64;
    let mut pos = 0;

    while pos + count <= total_bits {
        // Verifica se todos os bits no range estão livres
        let mut all_free = true;
        for i in 0..count {
            let bit = pos + i;
            let word_idx = bit / 64;
            let bit_idx = bit % 64;

            if (bitmap[word_idx] & (1 << bit_idx)) != 0 {
                all_free = false;
                break;
            }
        }

        if all_free {
            return Some(pos);
        }

        // Próxima posição alinhada
        pos += alignment;
    }

    None
}

/// Define range de bits como 1 (usado)
///
/// # Argumentos
///
/// * `bitmap` - Array mutável de u64
/// * `start` - Índice do primeiro bit
/// * `count` - Número de bits a setar
pub fn set_range(bitmap: &mut [u64], start: usize, count: usize) {
    for i in 0..count {
        let bit = start + i;
        let word_idx = bit / 64;
        let bit_idx = bit % 64;

        if word_idx < bitmap.len() {
            bitmap[word_idx] |= 1 << bit_idx;
        }
    }
}

/// Limpa range de bits para 0 (livre)
///
/// # Argumentos
///
/// * `bitmap` - Array mutável de u64
/// * `start` - Índice do primeiro bit
/// * `count` - Número de bits a limpar
pub fn clear_range(bitmap: &mut [u64], start: usize, count: usize) {
    for i in 0..count {
        let bit = start + i;
        let word_idx = bit / 64;
        let bit_idx = bit % 64;

        if word_idx < bitmap.len() {
            bitmap[word_idx] &= !(1 << bit_idx);
        }
    }
}

/// Define range de bits usando operações otimizadas
///
/// Mais eficiente que `set_range` para ranges grandes.
pub fn set_range_fast(bitmap: &mut [u64], start: usize, count: usize) {
    if count == 0 {
        return;
    }

    let end = start + count;
    let start_word = start / 64;
    let end_word = (end - 1) / 64;
    let start_bit = start % 64;
    let end_bit = (end - 1) % 64;

    if start_word == end_word {
        // Tudo em uma word
        let mask = ((1u64 << count) - 1) << start_bit;
        if start_word < bitmap.len() {
            bitmap[start_word] |= mask;
        }
    } else {
        // Primeira word (parcial)
        if start_word < bitmap.len() {
            bitmap[start_word] |= u64::MAX << start_bit;
        }

        // Words do meio (completas)
        for word_idx in (start_word + 1)..end_word {
            if word_idx < bitmap.len() {
                bitmap[word_idx] = u64::MAX;
            }
        }

        // Última word (parcial)
        if end_word < bitmap.len() {
            bitmap[end_word] |= (1u64 << (end_bit + 1)) - 1;
        }
    }
}

/// Limpa range de bits usando operações otimizadas
pub fn clear_range_fast(bitmap: &mut [u64], start: usize, count: usize) {
    if count == 0 {
        return;
    }

    let end = start + count;
    let start_word = start / 64;
    let end_word = (end - 1) / 64;
    let start_bit = start % 64;
    let end_bit = (end - 1) % 64;

    if start_word == end_word {
        // Tudo em uma word
        let mask = ((1u64 << count) - 1) << start_bit;
        if start_word < bitmap.len() {
            bitmap[start_word] &= !mask;
        }
    } else {
        // Primeira word (parcial)
        if start_word < bitmap.len() {
            bitmap[start_word] &= (1u64 << start_bit) - 1;
        }

        // Words do meio (completas)
        for word_idx in (start_word + 1)..end_word {
            if word_idx < bitmap.len() {
                bitmap[word_idx] = 0;
            }
        }

        // Última word (parcial)
        if end_word < bitmap.len() {
            bitmap[end_word] &= u64::MAX << (end_bit + 1);
        }
    }
}

/// Conta total de bits zero no bitmap
pub fn count_total_zeros(bitmap: &[u64]) -> usize {
    bitmap.iter().map(|w| w.count_zeros() as usize).sum()
}

/// Conta total de bits um no bitmap
pub fn count_total_ones(bitmap: &[u64]) -> usize {
    bitmap.iter().map(|w| w.count_ones() as usize).sum()
}

/// Verifica se bit está setado
#[inline]
pub fn is_set(bitmap: &[u64], bit: usize) -> bool {
    let word_idx = bit / 64;
    let bit_idx = bit % 64;

    if word_idx >= bitmap.len() {
        return false;
    }

    (bitmap[word_idx] & (1 << bit_idx)) != 0
}

/// Seta um bit
#[inline]
pub fn set_bit(bitmap: &mut [u64], bit: usize) {
    let word_idx = bit / 64;
    let bit_idx = bit % 64;

    if word_idx < bitmap.len() {
        bitmap[word_idx] |= 1 << bit_idx;
    }
}

/// Limpa um bit
#[inline]
pub fn clear_bit(bitmap: &mut [u64], bit: usize) {
    let word_idx = bit / 64;
    let bit_idx = bit % 64;

    if word_idx < bitmap.len() {
        bitmap[word_idx] &= !(1 << bit_idx);
    }
}

/// Toggle um bit
#[inline]
pub fn toggle_bit(bitmap: &mut [u64], bit: usize) {
    let word_idx = bit / 64;
    let bit_idx = bit % 64;

    if word_idx < bitmap.len() {
        bitmap[word_idx] ^= 1 << bit_idx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_first_zero() {
        assert_eq!(find_first_zero(0), Some(0));
        assert_eq!(find_first_zero(1), Some(1));
        assert_eq!(find_first_zero(0b1111), Some(4));
        assert_eq!(find_first_zero(u64::MAX), None);
    }

    #[test]
    fn test_find_contiguous_zeros() {
        let bitmap = [0b1111_0000u64]; // 4 zeros nos bits 4-7
        assert_eq!(find_contiguous_zeros(&bitmap, 4), Some(4));

        let bitmap = [u64::MAX, 0]; // segunda word toda livre
        assert_eq!(find_contiguous_zeros(&bitmap, 64), Some(64));
    }

    #[test]
    fn test_set_clear_range() {
        let mut bitmap = [0u64];

        set_range(&mut bitmap, 4, 4);
        assert_eq!(bitmap[0], 0b1111_0000);

        clear_range(&mut bitmap, 5, 2);
        assert_eq!(bitmap[0], 0b1001_0000);
    }

    #[test]
    fn test_count() {
        let bitmap = [0b1111_0000u64];
        assert_eq!(count_total_ones(&bitmap), 4);
        assert_eq!(count_total_zeros(&bitmap), 60);
    }
}
