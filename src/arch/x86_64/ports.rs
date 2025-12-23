//! Abstração de Portas de I/O (x86_64).
//!
//! Substitui o uso direto de assembly `in/out` nos drivers.

use core::arch::asm;
use core::marker::PhantomData;

/// Uma porta de I/O de leitura/escrita.
#[derive(Debug, Clone, Copy)]
pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T> Port<T> {
    /// Cria uma nova porta insegura.
    ///
    /// # Safety
    /// O caller deve garantir que a porta existe e é do tipo correto.
    pub const unsafe fn new(port: u16) -> Self {
        Self {
            port,
            phantom: PhantomData,
        }
    }
}

impl Port<u8> {
    /// Lê um byte da porta.
    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        asm!("in al, dx", out("al") value, in("dx") self.port, options(nomem, nostack, preserves_flags));
        value
    }

    /// Escreve um byte na porta.
    pub unsafe fn write(&mut self, value: u8) {
        asm!("out dx, al", in("dx") self.port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}
