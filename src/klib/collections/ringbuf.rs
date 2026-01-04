//! # Ring Buffer (Buffer Circular)
//!
//! Fila FIFO de tamanho fixo, útil para I/O e comunicação.
//! Não aloca memória após criação.

use core::mem::MaybeUninit;

/// Ring buffer de tamanho fixo.
///
/// ## Características:
/// - Tamanho fixo em tempo de compilação
/// - O(1) para push e pop
/// - Sem alocação após criação
pub struct RingBuffer<T, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    head: usize, // Próxima posição de leitura
    tail: usize, // Próxima posição de escrita
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Cria um ring buffer vazio.
    pub const fn new() -> Self {
        Self {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    /// Capacidade máxima.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Número de elementos.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Verifica se está vazio.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Verifica se está cheio.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Espaço disponível.
    #[inline]
    pub const fn available(&self) -> usize {
        N - self.len
    }

    /// Adiciona um elemento ao final.
    ///
    /// ## Retorno:
    /// - `Ok(())` se adicionou
    /// - `Err(item)` se estava cheio (retorna o item)
    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        self.buffer[self.tail] = MaybeUninit::new(item);
        self.tail = (self.tail + 1) % N;
        self.len += 1;
        Ok(())
    }

    /// Remove e retorna o primeiro elemento.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = unsafe { self.buffer[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(item)
    }

    /// Retorna referência ao primeiro elemento sem remover.
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        unsafe { Some(self.buffer[self.head].assume_init_ref()) }
    }

    /// Retorna referência mutável ao primeiro elemento sem remover.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        unsafe { Some(self.buffer[self.head].assume_init_mut()) }
    }

    /// Limpa o buffer.
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        // Drop de todos os elementos restantes
        while self.pop().is_some() {}
    }
}
