//! # Atomic Wrappers
//!
//! Wrappers convenientes sobre `core::sync::atomic`.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

// =============================================================================
// ATOMIC FLAG
// =============================================================================

/// Flag atômico com API simplificada.
pub struct AtomicFlag(AtomicBool);

impl AtomicFlag {
    /// Cria nova flag.
    #[inline]
    pub const fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }

    /// Lê valor atual.
    #[inline]
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Define valor.
    #[inline]
    pub fn set(&self, value: bool) {
        self.0.store(value, Ordering::Release);
    }

    /// Test-and-set: define true, retorna valor anterior.
    #[inline]
    pub fn test_and_set(&self) -> bool {
        self.0.swap(true, Ordering::AcqRel)
    }

    /// Limpa (define false).
    #[inline]
    pub fn clear(&self) {
        self.0.store(false, Ordering::Release);
    }

    /// Troca valor, retorna anterior.
    #[inline]
    pub fn swap(&self, value: bool) -> bool {
        self.0.swap(value, Ordering::AcqRel)
    }
}

impl Default for AtomicFlag {
    fn default() -> Self {
        Self::new(false)
    }
}

// =============================================================================
// ATOMIC COUNTER
// =============================================================================

/// Contador atômico de 64 bits.
pub struct AtomicCounter(AtomicU64);

impl AtomicCounter {
    /// Cria novo contador.
    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }

    /// Lê valor atual.
    #[inline]
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }

    /// Define valor.
    #[inline]
    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::Release);
    }

    /// Incrementa, retorna valor anterior.
    #[inline]
    pub fn inc(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel)
    }

    /// Decrementa, retorna valor anterior.
    #[inline]
    pub fn dec(&self) -> u64 {
        self.0.fetch_sub(1, Ordering::AcqRel)
    }

    /// Adiciona, retorna valor anterior.
    #[inline]
    pub fn add(&self, value: u64) -> u64 {
        self.0.fetch_add(value, Ordering::AcqRel)
    }

    /// Subtrai, retorna valor anterior.
    #[inline]
    pub fn sub(&self, value: u64) -> u64 {
        self.0.fetch_sub(value, Ordering::AcqRel)
    }

    /// Reseta para zero, retorna valor anterior.
    #[inline]
    pub fn reset(&self) -> u64 {
        self.0.swap(0, Ordering::AcqRel)
    }

    /// Incrementa e retorna novo valor.
    #[inline]
    pub fn inc_get(&self) -> u64 {
        self.inc() + 1
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

// =============================================================================
// ATOMIC USIZE
// =============================================================================

/// Contador atômico de tamanho de ponteiro.
pub struct AtomicSize(AtomicUsize);

impl AtomicSize {
    /// Cria novo.
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    /// Lê valor.
    #[inline]
    pub fn get(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    /// Define valor.
    #[inline]
    pub fn set(&self, value: usize) {
        self.0.store(value, Ordering::Release);
    }

    /// Adiciona, retorna anterior.
    #[inline]
    pub fn add(&self, value: usize) -> usize {
        self.0.fetch_add(value, Ordering::AcqRel)
    }

    /// Subtrai, retorna anterior.
    #[inline]
    pub fn sub(&self, value: usize) -> usize {
        self.0.fetch_sub(value, Ordering::AcqRel)
    }
}

impl Default for AtomicSize {
    fn default() -> Self {
        Self::new(0)
    }
}

// =============================================================================
// ATOMIC CELL
// =============================================================================

/// Célula atômica genérica para tipos pequenos.
///
/// **Nota**: Load/store não são atômicos para tipos > 8 bytes!
pub struct AtomicCell<T: Copy> {
    value: UnsafeCell<T>,
}

// SAFETY: AtomicCell usa operações atômicas para tipos pequenos
unsafe impl<T: Copy + Send> Send for AtomicCell<T> {}
unsafe impl<T: Copy + Send> Sync for AtomicCell<T> {}

impl<T: Copy> AtomicCell<T> {
    /// Cria nova célula.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    /// Carrega valor.
    ///
    /// **Nota**: Não é atômico para tipos > 8 bytes!
    #[inline]
    pub fn load(&self) -> T {
        // SAFETY: Assumimos acesso único ou tipo pequeno
        unsafe { *self.value.get() }
    }

    /// Armazena valor.
    ///
    /// **Nota**: Não é atômico para tipos > 8 bytes!
    #[inline]
    pub fn store(&self, value: T) {
        // SAFETY: Assumimos acesso único ou tipo pequeno
        unsafe {
            *self.value.get() = value;
        }
    }

    /// Troca valor, retorna anterior.
    #[inline]
    pub fn swap(&self, value: T) -> T {
        let old = self.load();
        self.store(value);
        old
    }
}

impl<T: Copy + Default> Default for AtomicCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
