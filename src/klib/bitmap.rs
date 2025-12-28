//! Bitmap genérico

/// Bitmap para gerenciamento de bits
pub struct Bitmap<'a> {
    data: &'a mut [u64],
    len: usize,
}

impl<'a> Bitmap<'a> {
    /// Cria bitmap sobre slice existente
    pub fn new(data: &'a mut [u64], bits: usize) -> Self {
        Self { data, len: bits }
    }
    
    /// Define um bit
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] |= 1 << bit;
    }
    
    /// Limpa um bit
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] &= !(1 << bit);
    }
    
    /// Testa um bit
    pub fn test(&self, index: usize) -> bool {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        (self.data[word] & (1 << bit)) != 0
    }
    
    /// Encontra primeiro bit livre (0)
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
}
