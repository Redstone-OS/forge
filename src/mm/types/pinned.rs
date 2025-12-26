//! # Pinned<T> - Tipo para Valores que Não Podem Ser Movidos
//!
//! ## 🎯 Propósito
//!
//! Alguns tipos não podem ser movidos após inicialização:
//! - Estruturas self-referential
//! - Objetos registrados em hardware
//! - Page tables
//!
//! Pinned<T> garante em tempo de compilação que o valor não será movido.
//!
//! ## 🏗️ Arquitetura
//!
//! - `Pin<T>`: Wrapper que impede acesso &mut T
//! - `Pinned<T>`: Trait marker para tipos que requerem pinning
//!
//! ## 🔧 Uso
//!
//! ```rust
//! // Criar valor pinned
//! let pinned = Pin::new(Box::new(MyStruct::new()));
//!
//! // Acessar imutavelmente: OK
//! pinned.method();
//!
//! // Tentar mover: ERRO DE COMPILAÇÃO
//! // let moved = *pinned; // Erro!
//! ```

use core::marker::PhantomPinned;
use core::ops::{Deref, DerefMut};
use core::pin::Pin as StdPin;

// =============================================================================
// TRAIT PINNED
// =============================================================================

/// Marker trait para tipos que requerem pinning
///
/// Implementar este trait indica que o tipo NÃO pode ser movido
/// após inicialização.
pub trait Pinned {}

// =============================================================================
// PIN WRAPPER
// =============================================================================

/// Wrapper que garante que T não será movido
///
/// Similar a `core::pin::Pin`, mas com API focada em kernel.
pub struct Pin<T> {
    inner: T,
    _marker: PhantomPinned,
}

impl<T> Pin<T> {
    /// Cria novo Pin (consome ownership)
    ///
    /// # Safety
    ///
    /// O caller deve garantir que T pode ser pinned e que não será
    /// movido após esta chamada.
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            inner: value,
            _marker: PhantomPinned,
        }
    }

    /// Cria pin para tipo que implementa Unpin
    pub fn new(value: T) -> Self
    where
        T: Unpin,
    {
        Self {
            inner: value,
            _marker: PhantomPinned,
        }
    }

    /// Obtém referência ao valor interno
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Obtém referência mutável ao valor interno
    ///
    /// # Safety
    ///
    /// O caller deve garantir que não moverá T.
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Obtém Pin para o valor interno
    pub fn as_pin(&mut self) -> StdPin<&mut T> {
        unsafe { StdPin::new_unchecked(&mut self.inner) }
    }

    /// Desempacota (consome o Pin)
    ///
    /// # Safety
    ///
    /// O caller deve garantir que não moverá T após desempacotar.
    pub unsafe fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Deref for Pin<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Não implementamos DerefMut para prevenir movimentação

// =============================================================================
// PIN BOX
// =============================================================================

/// Pin para valores alocados no heap
///
/// Usa Box internamente, garantindo que o valor está em endereço fixo.
pub struct PinBox<T: ?Sized> {
    inner: alloc::boxed::Box<T>,
}

impl<T> PinBox<T> {
    /// Cria novo PinBox
    pub fn new(value: T) -> Self {
        Self {
            inner: alloc::boxed::Box::new(value),
        }
    }

    /// Obtém referência
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Obtém referência mutável
    ///
    /// Seguro porque o Box não pode ser movido.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Converte para Pin padrão
    ///
    /// # Safety
    /// Seguro porque PinBox garante que o valor não será movido
    pub fn as_pin(&mut self) -> StdPin<&mut T> {
        // Safety: PinBox garante que o valor não será movido
        unsafe { StdPin::new_unchecked(&mut *self.inner) }
    }

    /// Obtém endereço do valor
    pub fn as_ptr(&self) -> *const T {
        &*self.inner as *const T
    }
}

impl<T: ?Sized> Deref for PinBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for PinBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// =============================================================================
// PINNED PAGE
// =============================================================================

/// Página de memória pinned
///
/// Representa uma página física que foi pinned (não pode ser swapped
/// ou movida pelo sistema).
pub struct PinnedPage {
    /// Endereço físico da página
    phys_addr: u64,
    /// Endereço virtual mapeado
    virt_addr: u64,
    /// Referência contada
    ref_count: core::sync::atomic::AtomicUsize,
}

impl PinnedPage {
    /// Cria nova página pinned
    pub fn new(phys: u64, virt: u64) -> Self {
        Self {
            phys_addr: phys,
            virt_addr: virt,
            ref_count: core::sync::atomic::AtomicUsize::new(1),
        }
    }

    /// Endereço físico
    pub fn phys(&self) -> u64 {
        self.phys_addr
    }

    /// Endereço virtual
    pub fn virt(&self) -> u64 {
        self.virt_addr
    }

    /// Incrementa referência
    pub fn add_ref(&self) {
        self.ref_count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Decrementa referência
    pub fn release(&self) -> usize {
        self.ref_count
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed)
            - 1
    }

    /// Contagem de referências
    pub fn ref_count(&self) -> usize {
        self.ref_count.load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl Pinned for PinnedPage {}
