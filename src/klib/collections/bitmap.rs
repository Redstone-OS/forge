//! # Bitmap Genérico
//!
//! Estrutura para gerenciamento eficiente de arrays de bits.
//! Usado pelo PMM para rastrear frames, gerenciar IRQs, etc.
//!
//! ## Performance:
//! - `set`/`clear`/`test`: O(1)
//! - `find_first_zero`: O(N/64) - otimizado com trailing_ones
//! - `find_contiguous_zeros`: O(N)

use super::super::primitives::bits;

/// Bitmap para gerenciamento de bits.
///
/// Opera sobre um slice de `u64` fornecido externamente.
/// Não aloca memória própria.
pub struct Bitmap<'a> {
    data: &'a mut [u64],
    len: usize, // Número total de bits
}

impl<'a> Bitmap<'a> {
    /// Cria bitmap sobre slice existente.
    ///
    /// ## Parâmetros:
    /// - `data`: Slice de u64 para armazenar os bits
    /// - `bits`: Número total de bits a gerenciar (pode ser < data.len() * 64)
    pub fn new(data: &'a mut [u64], bits: usize) -> Self {
        debug_assert!(bits <= data.len() * 64);
        Self { data, len: bits }
    }

    /// Número total de bits.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Verifica se está vazio.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Define um bit como 1.
    #[inline]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] |= 1u64 << bit;
    }

    /// Limpa um bit para 0.
    #[inline]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] &= !(1u64 << bit);
    }

    /// Testa valor de um bit.
    #[inline]
    pub fn test(&self, index: usize) -> bool {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        (self.data[word] & (1u64 << bit)) != 0
    }

    /// Testa bit com verificação de bounds (retorna None se fora).
    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let word = index / 64;
        let bit = index % 64;
        Some((self.data[word] & (1u64 << bit)) != 0)
    }

    /// Define range de bits como 1.
    pub fn set_range(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            if i < self.len {
                self.set(i);
            }
        }
    }

    /// Limpa range de bits para 0.
    pub fn clear_range(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            if i < self.len {
                self.clear(i);
            }
        }
    }

    /// Encontra primeiro bit 0 (primeiro recurso livre).
    ///
    /// Otimizado usando `trailing_ones` que compila para `bsf`.
    pub fn find_first_zero(&self) -> Option<usize> {
        for (i, &word) in self.data.iter().enumerate() {
            if word != u64::MAX {
                let bit = word.trailing_ones() as usize;
                let index = i * 64 + bit;
                if index < self.len {
                    return Some(index);
                }
            }
        }
        None
    }

    /// Encontra primeiro bit 1.
    pub fn find_first_one(&self) -> Option<usize> {
        for (i, &word) in self.data.iter().enumerate() {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let index = i * 64 + bit;
                if index < self.len {
                    return Some(index);
                }
            }
        }
        None
    }

    /// Encontra N bits 0 contíguos.
    ///
    /// Útil para alocar múltiplas páginas contíguas.
    pub fn find_contiguous_zeros(&self, count: usize) -> Option<usize> {
        if count == 0 {
            return Some(0);
        }
        if count == 1 {
            return self.find_first_zero();
        }

        let mut run_start = 0;
        let mut run_len = 0;

        for i in 0..self.len {
            if !self.test(i) {
                if run_len == 0 {
                    run_start = i;
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

    /// Conta bits 1.
    pub fn count_ones(&self) -> usize {
        let mut count = 0;
        for &word in self.data.iter() {
            count += bits::popcount(word) as usize;
        }
        // Ajustar para bits que não fazem parte do bitmap
        let extra_bits = self.data.len() * 64 - self.len;
        if extra_bits > 0 && !self.data.is_empty() {
            let last_word = self.data.len() - 1;
            let mask = !((1u64 << (64 - extra_bits)) - 1);
            count -= (self.data[last_word] & mask).count_ones() as usize;
        }
        count
    }

    /// Conta bits 0.
    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.len - self.count_ones()
    }

    /// Limpa todos os bits para 0.
    pub fn clear_all(&mut self) {
        for word in self.data.iter_mut() {
            *word = 0;
        }
    }

    /// Define todos os bits como 1.
    pub fn set_all(&mut self) {
        for word in self.data.iter_mut() {
            *word = u64::MAX;
        }
        // Limpa bits extras no último word
        let extra_bits = self.data.len() * 64 - self.len;
        if extra_bits > 0 && !self.data.is_empty() {
            let last = self.data.len() - 1;
            self.data[last] &= (1u64 << (64 - extra_bits)) - 1;
        }
    }
}
