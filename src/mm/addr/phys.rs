use crate::mm::config::{align_down, align_up, is_aligned, PAGE_SIZE};
use core::fmt;

/// Endereço físico (wrapper type-safe)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Cria novo endereço físico
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

    /// Alinha para baixo (múltiplo de align)
    #[inline]
    pub fn align_down(self, align: u64) -> Self {
        Self(align_down(self.0 as usize, align as usize) as u64)
    }

    /// Alinha para cima (múltiplo de align)
    #[inline]
    pub fn align_up(self, align: u64) -> Self {
        Self(align_up(self.0 as usize, align as usize) as u64)
    }

    /// Verifica alinhamento
    #[inline]
    pub fn is_aligned(self, align: u64) -> bool {
        is_aligned(self.0 as usize, align as usize)
    }

    /// Verifica se é nulo
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Adiciona offset
    #[inline]
    pub fn add(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr({:#x})", self.0)
    }
}

impl fmt::Binary for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}
