//! # Bitmap de Frames
//!
//! Helpers para manipulação de bitmap.

/// Encontra primeiro bit zero
#[inline]
pub fn find_first_zero(bitmap: u64) -> Option<usize> {
    let inverted = !bitmap;
    if inverted == 0 {
        return None;
    }
    Some(inverted.trailing_zeros() as usize)
}

/// Encontra N bits zero contíguos
pub fn find_contiguous_zeros(bitmap: &[u64], count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    let mut run_start = 0;
    let mut run_len = 0;

    for (word_idx, &word) in bitmap.iter().enumerate() {
        for bit in 0..64 {
            let global_bit = word_idx * 64 + bit;
            if (word & (1 << bit)) == 0 {
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
    }
    None
}

/// Define range de bits
pub fn set_range(bitmap: &mut [u64], start: usize, count: usize) {
    for i in start..start + count {
        let word = i / 64;
        let bit = i % 64;
        if word < bitmap.len() {
            bitmap[word] |= 1 << bit;
        }
    }
}

/// Limpa range de bits
pub fn clear_range(bitmap: &mut [u64], start: usize, count: usize) {
    for i in start..start + count {
        let word = i / 64;
        let bit = i % 64;
        if word < bitmap.len() {
            bitmap[word] &= !(1 << bit);
        }
    }
}
