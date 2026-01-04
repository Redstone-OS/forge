//! # FNV-1a Hash
//!
//! Implementação do algoritmo FNV-1a (Fowler–Noll–Vo).
//! Hash rápido e de boa distribuição para uso geral.
//!
//! ## Características:
//! - Não-criptográfico (não usar para segurança)
//! - Rápido
//! - Boa distribuição
//! - Sem dependências externas

use core::hash::Hasher;

/// Offset inicial FNV-1a para 64 bits.
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// Prime FNV-1a para 64 bits.
const FNV_PRIME: u64 = 0x100000001b3;

/// Hasher FNV-1a de 64 bits.
///
/// Implementa `core::hash::Hasher` para uso com `Hash` trait.
pub struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    /// Cria um novo hasher com offset padrão.
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    /// Cria um hasher com seed customizada.
    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Default for FnvHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for FnvHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }
}

/// Hash rápido de um slice de bytes usando FNV-1a.
///
/// Função de conveniência quando não precisa do trait Hasher.
#[inline]
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hasher = FnvHasher::new();
    hasher.write(data);
    hasher.finish()
}

/// Hash de string usando FNV-1a.
#[inline]
pub fn fnv1a_hash_str(s: &str) -> u64 {
    fnv1a_hash(s.as_bytes())
}

/// Hash de u64 usando FNV-1a.
#[inline]
pub fn fnv1a_hash_u64(value: u64) -> u64 {
    fnv1a_hash(&value.to_le_bytes())
}

/// Hash de usize usando FNV-1a.
#[inline]
pub fn fnv1a_hash_usize(value: usize) -> u64 {
    fnv1a_hash(&value.to_le_bytes())
}

/// Combina dois hashes.
#[inline]
pub fn hash_combine(h1: u64, h2: u64) -> u64 {
    let mut hasher = FnvHasher::with_seed(h1);
    hasher.write(&h2.to_le_bytes());
    hasher.finish()
}
