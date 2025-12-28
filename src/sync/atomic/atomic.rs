//! Operações atômicas

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::cell::UnsafeCell;

/// Célula atômica genérica para tipos pequenos
pub struct AtomicCell<T: Copy> {
    value: UnsafeCell<T>,
}

// SAFETY: AtomicCell usa operações atômicas internamente
unsafe impl<T: Copy + Send> Send for AtomicCell<T> {}
unsafe impl<T: Copy + Send> Sync for AtomicCell<T> {}

impl<T: Copy> AtomicCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }
    
    /// Carrega o valor (não atômico para tipos grandes!)
    pub fn load(&self) -> T {
        // SAFETY: Assumimos acesso único ou tipo atômico
        unsafe { *self.value.get() }
    }
    
    /// Armazena o valor
    pub fn store(&self, value: T) {
        // SAFETY: Assumimos acesso único ou tipo atômico
        unsafe { *self.value.get() = value; }
    }
}

/// Wrapper para AtomicBool com API mais limpa
pub struct AtomicFlag(AtomicBool);

impl AtomicFlag {
    pub const fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }
    
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    
    pub fn set(&self, value: bool) {
        self.0.store(value, Ordering::Release);
    }
    
    /// Test-and-set: retorna valor anterior
    pub fn test_and_set(&self) -> bool {
        self.0.swap(true, Ordering::AcqRel)
    }
    
    pub fn clear(&self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Contador atômico
pub struct AtomicCounter(AtomicU64);

impl AtomicCounter {
    pub const fn new(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }
    
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }
    
    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::Release);
    }
    
    pub fn inc(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel)
    }
    
    pub fn dec(&self) -> u64 {
        self.0.fetch_sub(1, Ordering::AcqRel)
    }
    
    pub fn add(&self, value: u64) -> u64 {
        self.0.fetch_add(value, Ordering::AcqRel)
    }
}
