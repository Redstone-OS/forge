//! Bitmap Estático/Dinâmico.
//!
//! Útil para gerenciamento de recursos (PMM, PIDs, Inodes).


pub struct Bitmap<'a> {
    data: &'a mut [u64],
    len: usize, // em bits
}

impl<'a> Bitmap<'a> {
    pub fn new(data: &'a mut [u64], len_bits: usize) -> Self {
        Self {
            data,
            len: len_bits,
        }
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        if idx >= self.len {
            return;
        }

        let word_idx = idx / 64;
        let bit_idx = idx % 64;

        if value {
            self.data[word_idx] |= 1 << bit_idx;
        } else {
            self.data[word_idx] &= !(1 << bit_idx);
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        if idx >= self.len {
            return false;
        }

        let word_idx = idx / 64;
        let bit_idx = idx % 64;

        (self.data[word_idx] & (1 << bit_idx)) != 0
    }

    /// Encontra o primeiro bit com valor `val` (0 ou 1).
    pub fn find_first(&self, val: bool) -> Option<usize> {
        for (i, &word) in self.data.iter().enumerate() {
            // Se procuramos 0 (livre) e a palavra não é tudo 1 (cheia)
            // Ou se procuramos 1 (usado) e a palavra não é tudo 0 (vazia)
            let interesting = if val { word != 0 } else { word != u64::MAX };

            if interesting {
                for bit in 0..64 {
                    let idx = i * 64 + bit;
                    if idx >= self.len {
                        return None;
                    }

                    let bit_val = (word & (1 << bit)) != 0;
                    if bit_val == val {
                        return Some(idx);
                    }
                }
            }
        }
        None
    }

    pub fn fill(&mut self, val: bool) {
        let fill = if val { u64::MAX } else { 0 };
        self.data.fill(fill);
    }
}
