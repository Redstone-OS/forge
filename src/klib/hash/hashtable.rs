/// Arquivo: klib/hash/hashtable.rs
///
/// Propósito: Tabela Hash (Dicionário).
/// Mapeia Chaves -> Valores usando uma função de hash para acesso O(1) médio.
///
/// Detalhes de Implementação:
/// - Encadeamento para colisões (Vec de Buckets).
/// - Função de Hash simples interna ou Trait Hash (vamos usar Hash trait do core).

//! Hash Table

use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
// Nota: Em no_std, BuildHasherDefault não está sempre disponível facilmente sem std, 
// então implementamos um Hasher simples FNV-1a.

pub struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    fn new() -> Self {
        Self { state: 0xcbf29ce484222325 }
    }
}

impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(0x100000001b3);
        }
    }
    
    fn finish(&self) -> u64 {
        self.state
    }
}

struct Entry<K, V> {
    key: K,
    value: V,
}

pub struct HashTable<K, V> {
    buckets: Vec<Vec<Entry<K, V>>>,
    len: usize,
}

impl<K: Hash + Eq, V> HashTable<K, V> {
    pub fn new(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }
        Self {
            buckets,
            len: 0,
        }
    }

    fn get_bucket_index(&self, key: &K) -> usize {
        let mut hasher = FnvHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.buckets.len()
    }

    pub fn insert(&mut self, key: K, value: V) {
        let index = self.get_bucket_index(&key);
        let bucket = &mut self.buckets[index];
        
        // Verifica se chave já existe para atualizar
        for entry in bucket.iter_mut() {
            if entry.key == key {
                entry.value = value;
                return;
            }
        }
        
        bucket.push(Entry { key, value });
        self.len += 1;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.get_bucket_index(key);
        let bucket = &self.buckets[index];
        
        for entry in bucket {
            if entry.key == *key {
                return Some(&entry.value);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.get_bucket_index(key);
        let bucket = &mut self.buckets[index];
        
        if let Some(pos) = bucket.iter().position(|e| e.key == *key) {
            self.len -= 1;
            return Some(bucket.remove(pos).value);
        }
        None
    }
}
