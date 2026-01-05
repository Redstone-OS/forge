//! # Tipos de Endereço
//!
//! Define `PhysAddr` e `VirtAddr` como tipos distintos para evitar confusão
//! entre endereços físicos e virtuais. Isso é crucial para segurança de memória.
//!
//! ## Uso
//!
//! ```rust
//! use crate::rmm::{PhysAddr, VirtAddr};
//!
//! let phys = PhysAddr::new(0x1000);
//! let virt = VirtAddr::new(0xFFFF_8000_0000_1000);
//!
//! // Conversão via HHDM
//! let virt = phys.to_virt();
//! let phys = virt.to_phys();
//! ```
//!
//! ## Alinhamento
//!
//! As funções `align_up` e `align_down` auxiliam no alinhamento de endereços.

use core::fmt;
use core::ops::{Add, Sub};

use super::config::{PAGE_MASK, PAGE_SIZE};

// =============================================================================
// PhysAddr
// =============================================================================

/// Endereço físico de memória.
///
/// Representa um endereço na RAM física real. Não pode ser desreferenciado
/// diretamente pela CPU; deve ser convertido para virtual via HHDM.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Cria um novo endereço físico
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Cria endereço físico zero
    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Retorna o valor bruto do endereço
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Verifica se o endereço é zero
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Verifica se o endereço está alinhado a `align` bytes
    #[inline]
    pub const fn is_aligned(&self, align: usize) -> bool {
        self.0 % align as u64 == 0
    }

    /// Verifica se está alinhado a página
    #[inline]
    pub const fn is_page_aligned(&self) -> bool {
        self.0 & (PAGE_SIZE as u64 - 1) == 0
    }

    /// Alinha o endereço para cima
    #[inline]
    pub const fn align_up(&self, align: usize) -> Self {
        Self(align_up_u64(self.0, align as u64))
    }

    /// Alinha o endereço para baixo
    #[inline]
    pub const fn align_down(&self, align: usize) -> Self {
        Self(align_down_u64(self.0, align as u64))
    }

    /// Alinha para página
    #[inline]
    pub const fn page_align_up(&self) -> Self {
        Self(align_up_u64(self.0, PAGE_SIZE as u64))
    }

    /// Alinha para página (baixo)
    #[inline]
    pub const fn page_align_down(&self) -> Self {
        Self(self.0 & PAGE_MASK)
    }

    /// Retorna o offset dentro da página
    #[inline]
    pub const fn page_offset(&self) -> usize {
        (self.0 & (PAGE_SIZE as u64 - 1)) as usize
    }

    /// Retorna o número do frame (page frame number)
    #[inline]
    pub const fn pfn(&self) -> u64 {
        self.0 / PAGE_SIZE as u64
    }

    /// Cria PhysAddr a partir do PFN
    #[inline]
    pub const fn from_pfn(pfn: u64) -> Self {
        Self(pfn * PAGE_SIZE as u64)
    }

    /// Converte para endereço virtual via HHDM
    ///
    /// # Panics
    ///
    /// Panic se HHDM não estiver inicializado
    #[inline]
    pub fn to_virt(&self) -> VirtAddr {
        super::virt::hhdm::phys_to_virt_addr(*self)
    }

    /// Converte para ponteiro via HHDM
    ///
    /// # Safety
    ///
    /// O caller deve garantir que:
    /// - HHDM está inicializado
    /// - O endereço físico é válido
    /// - O tipo T tem layout compatível
    #[inline]
    pub unsafe fn as_ptr<T>(&self) -> *const T {
        self.to_virt().as_ptr()
    }

    /// Converte para ponteiro mutável via HHDM
    ///
    /// # Safety
    ///
    /// Mesmas condições de `as_ptr`, mais acesso exclusivo
    #[inline]
    pub unsafe fn as_mut_ptr<T>(&self) -> *mut T {
        self.to_virt().as_mut_ptr()
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs as u64)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self {
        Self(self.0 - rhs)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysAddr) -> u64 {
        self.0 - rhs.0
    }
}

impl From<u64> for PhysAddr {
    #[inline]
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<PhysAddr> for u64 {
    #[inline]
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

// =============================================================================
// VirtAddr
// =============================================================================

/// Endereço virtual de memória.
///
/// Representa um endereço no espaço de endereçamento virtual da CPU.
/// Pode ser desreferenciado (após verificar validade).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Cria um novo endereço virtual
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Cria endereço virtual zero
    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Retorna o valor bruto do endereço
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Verifica se o endereço é zero
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Verifica se está alinhado
    #[inline]
    pub const fn is_aligned(&self, align: usize) -> bool {
        self.0 % align as u64 == 0
    }

    /// Verifica se está alinhado a página
    #[inline]
    pub const fn is_page_aligned(&self) -> bool {
        self.0 & (PAGE_SIZE as u64 - 1) == 0
    }

    /// Alinha para cima
    #[inline]
    pub const fn align_up(&self, align: usize) -> Self {
        Self(align_up_u64(self.0, align as u64))
    }

    /// Alinha para baixo
    #[inline]
    pub const fn align_down(&self, align: usize) -> Self {
        Self(align_down_u64(self.0, align as u64))
    }

    /// Alinha para página (cima)
    #[inline]
    pub const fn page_align_up(&self) -> Self {
        Self(align_up_u64(self.0, PAGE_SIZE as u64))
    }

    /// Alinha para página (baixo)
    #[inline]
    pub const fn page_align_down(&self) -> Self {
        Self(self.0 & PAGE_MASK)
    }

    /// Retorna offset dentro da página
    #[inline]
    pub const fn page_offset(&self) -> usize {
        (self.0 & (PAGE_SIZE as u64 - 1)) as usize
    }

    /// Verifica se é endereço de userspace (lower half)
    #[inline]
    pub const fn is_user(&self) -> bool {
        self.0 < 0x0000_8000_0000_0000
    }

    /// Verifica se é endereço de kernel (higher half)
    #[inline]
    pub const fn is_kernel(&self) -> bool {
        self.0 >= 0xFFFF_8000_0000_0000
    }

    /// Verifica se é endereço canônico válido
    #[inline]
    pub const fn is_canonical(&self) -> bool {
        // Em x86_64, bits 48-63 devem ser todos 0 ou todos 1
        let sign_ext = (self.0 as i64) >> 47;
        sign_ext == 0 || sign_ext == -1
    }

    /// Converte para endereço físico via HHDM (se for endereço HHDM)
    ///
    /// Retorna None se não for endereço HHDM
    #[inline]
    pub fn to_phys(&self) -> Option<PhysAddr> {
        super::virt::hhdm::virt_to_phys_addr(*self)
    }

    /// Converte para ponteiro
    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Converte para ponteiro mutável
    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Extrai índice no PML4 (bits 39-47)
    #[inline]
    pub const fn pml4_index(&self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }

    /// Extrai índice no PDPT (bits 30-38)
    #[inline]
    pub const fn pdpt_index(&self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }

    /// Extrai índice no PD (bits 21-29)
    #[inline]
    pub const fn pd_index(&self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }

    /// Extrai índice no PT (bits 12-20)
    #[inline]
    pub const fn pt_index(&self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs as u64)
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self {
        Self(self.0 - rhs)
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: VirtAddr) -> u64 {
        self.0 - rhs.0
    }
}

impl From<u64> for VirtAddr {
    #[inline]
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<VirtAddr> for u64 {
    #[inline]
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}

impl<T> From<*const T> for VirtAddr {
    #[inline]
    fn from(p: *const T) -> Self {
        Self(p as u64)
    }
}

impl<T> From<*mut T> for VirtAddr {
    #[inline]
    fn from(p: *mut T) -> Self {
        Self(p as u64)
    }
}

// =============================================================================
// Funções Auxiliares
// =============================================================================

/// Alinha valor para cima
#[inline]
pub const fn align_up_u64(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

/// Alinha valor para baixo
#[inline]
pub const fn align_down_u64(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

/// Alinha usize para cima
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Alinha usize para baixo
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

/// Verifica se valor é potência de 2
#[inline]
pub const fn is_power_of_two(value: usize) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

/// Calcula número de páginas para um tamanho
#[inline]
pub const fn pages_for_size(size: usize) -> usize {
    align_up(size, PAGE_SIZE) / PAGE_SIZE
}
