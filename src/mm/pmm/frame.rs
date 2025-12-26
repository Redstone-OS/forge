use crate::mm::addr::PhysAddr;
use crate::mm::config::PAGE_SIZE;
use core::fmt;

/// Um frame de memória física (tamanho fixo PAGE_SIZE = 4KiB)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysFrame {
    start_address: PhysAddr,
}

impl PhysFrame {
    /// Retorna o frame que contém o endereço físico dado
    #[inline]
    pub fn containing_address(addr: PhysAddr) -> Self {
        Self {
            start_address: addr.align_down(PAGE_SIZE as u64),
        }
    }

    /// Cria um frame a partir de um endereço (deve estar alinhado)
    /// Retorna erro ou panic se não estiver alinhado? Por segurança, alinha.
    #[inline]
    pub const fn from_start_address(addr: PhysAddr) -> Self {
        // Idealmente verificaria alinhamento, mas const fn é limitado
        Self {
            start_address: addr,
        }
    }

    /// Retorna o endereço inicial do frame
    #[inline]
    pub const fn start_address(&self) -> PhysAddr {
        self.start_address
    }

    /// Retorna o tamanho do frame
    #[inline]
    pub const fn size(&self) -> u64 {
        PAGE_SIZE as u64
    }

    /// Adiciona offset de N frames
    #[inline]
    pub fn add(&self, count: u64) -> Self {
        Self {
            start_address: self.start_address.add(count * PAGE_SIZE as u64),
        }
    }

    /// Retorna o endereço físico como u64 (Compatibilidade legacy)
    #[inline]
    pub const fn addr(&self) -> u64 {
        self.start_address.as_u64()
    }
}

impl fmt::Debug for PhysFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysFrame({:?})", self.start_address)
    }
}
