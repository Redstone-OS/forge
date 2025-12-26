use crate::mm::config::{align_down, align_up, is_aligned};
use core::fmt;

/// Endereço virtual (wrapper type-safe)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Cria novo endereço virtual
    /// Tenta normalizar canonical form (sign extension) seria ideal, mas por enquanto simples
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Retorna o valor interno como u64
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Retorna o valor interno como usize
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Retorna ponteiro raw const
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Retorna ponteiro raw mut
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Alinha para baixo
    #[inline]
    pub fn align_down(self, align: u64) -> Self {
        Self(align_down(self.0 as usize, align as usize) as u64)
    }

    /// Alinha para cima
    #[inline]
    pub fn align_up(self, align: u64) -> Self {
        Self(align_up(self.0 as usize, align as usize) as u64)
    }

    /// Verifica alinhamento
    #[inline]
    pub fn is_aligned(self, align: u64) -> bool {
        is_aligned(self.0 as usize, align as usize)
    }

    /// Adiciona offset
    #[inline]
    pub fn add(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}
