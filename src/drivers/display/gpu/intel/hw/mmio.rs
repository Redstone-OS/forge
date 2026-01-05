//! # MMIO Access
//!
//! Abstração para acesso MMIO aos registros do GPU Intel.
//! O GPU Intel usa memory-mapped I/O no BAR0 (GTTMMADR).

use crate::rmm::addr::VirtAddr;

// =============================================================================
// MMIO WRAPPER
// =============================================================================

/// Wrapper para acesso MMIO ao GPU Intel.
#[derive(Clone)]
pub struct Mmio {
    /// Endereço virtual base do MMIO.
    base: VirtAddr,
    /// Tamanho da região MMIO.
    size: usize,
}

impl Mmio {
    /// Cria novo wrapper MMIO.
    ///
    /// # Safety
    /// O caller deve garantir que `base` é um endereço válido mapeado.
    pub unsafe fn new(base: VirtAddr, size: usize) -> Self {
        Self { base, size }
    }

    /// Lê um registrador de 32 bits.
    #[inline]
    pub fn read32(&self, offset: u32) -> u32 {
        debug_assert!((offset as usize) + 4 <= self.size);
        let addr = self.base.as_u64() + offset as u64;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Escreve um registrador de 32 bits.
    #[inline]
    pub fn write32(&self, offset: u32, value: u32) {
        debug_assert!((offset as usize) + 4 <= self.size);
        let addr = self.base.as_u64() + offset as u64;
        unsafe { core::ptr::write_volatile(addr as *mut u32, value) }
    }

    /// Lê um registrador de 64 bits.
    #[inline]
    pub fn read64(&self, offset: u32) -> u64 {
        debug_assert!((offset as usize) + 8 <= self.size);
        let addr = self.base.as_u64() + offset as u64;
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }

    /// Escreve um registrador de 64 bits.
    #[inline]
    pub fn write64(&self, offset: u32, value: u64) {
        debug_assert!((offset as usize) + 8 <= self.size);
        let addr = self.base.as_u64() + offset as u64;
        unsafe { core::ptr::write_volatile(addr as *mut u64, value) }
    }

    /// Modifica bits específicos de um registrador.
    #[inline]
    pub fn set_bits(&self, offset: u32, mask: u32) {
        let val = self.read32(offset);
        self.write32(offset, val | mask);
    }

    /// Limpa bits específicos de um registrador.
    #[inline]
    pub fn clear_bits(&self, offset: u32, mask: u32) {
        let val = self.read32(offset);
        self.write32(offset, val & !mask);
    }

    /// Espera até que bits específicos sejam setados.
    pub fn wait_for_bits(&self, offset: u32, mask: u32, timeout_us: u32) -> bool {
        for _ in 0..timeout_us {
            if (self.read32(offset) & mask) == mask {
                return true;
            }
            // Busy wait ~1us
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }
        false
    }

    /// Retorna endereço base.
    pub fn base(&self) -> VirtAddr {
        self.base
    }

    /// Retorna tamanho.
    pub fn size(&self) -> usize {
        self.size
    }
}
