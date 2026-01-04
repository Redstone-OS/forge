//! # Lista Intrusiva Duplamente Encadeada
//!
//! Lista onde os ponteiros de ligação são parte da própria struct.
//! Zero alocação para operações de add/remove.
//!
//! ## Uso Principal:
//! - RunQueue do scheduler
//! - Wait queues
//! - Timer lists
//!
//! ## Comparação com Lista Normal:
//! ```text
//! Lista NORMAL (alloc::LinkedList):
//!   [Node] -> [Node] -> [Node]
//!     ↑         ↑         ↑
//!   alocado   alocado   alocado   <- Cada push aloca!
//!
//! Lista INTRUSIVA:
//!   [Task com pointers] <-> [Task com pointers] <-> [Task com pointers]
//!                          Nenhuma alocação extra!
//! ```

use core::ptr;

/// Trait que a struct deve implementar para ser linkável.
///
/// ## Exemplo:
/// ```rust
/// struct Task {
///     pid: u32,
///     run_next: *mut Task,
///     run_prev: *mut Task,
/// }
///
/// impl Linked for Task {
///     fn next(&self) -> *mut Self { self.run_next }
///     fn prev(&self) -> *mut Self { self.run_prev }
///     fn set_next(&mut self, n: *mut Self) { self.run_next = n; }
///     fn set_prev(&mut self, p: *mut Self) { self.run_prev = p; }
/// }
/// ```
pub trait Linked: Sized {
    /// Retorna ponteiro para o próximo elemento.
    fn next(&self) -> *mut Self;

    /// Retorna ponteiro para o elemento anterior.
    fn prev(&self) -> *mut Self;

    /// Define o próximo elemento.
    fn set_next(&mut self, next: *mut Self);

    /// Define o elemento anterior.
    fn set_prev(&mut self, prev: *mut Self);

    /// Verifica se o item está em alguma lista.
    fn is_linked(&self) -> bool {
        !self.next().is_null() || !self.prev().is_null()
    }

    /// Remove os links (limpa ponteiros).
    fn unlink(&mut self) {
        self.set_next(ptr::null_mut());
        self.set_prev(ptr::null_mut());
    }
}

/// Lista duplamente encadeada intrusiva.
///
/// ## Invariantes:
/// - Se a lista não está vazia, head e tail não são null
/// - Se a lista tem um elemento, head == tail
/// - O prev do head é null
/// - O next do tail é null
pub struct IntrusiveList<T: Linked> {
    head: *mut T,
    tail: *mut T,
    len: usize,
}

// Safety: A lista pode ser enviada entre threads se T também pode
unsafe impl<T: Linked + Send> Send for IntrusiveList<T> {}

// Safety: A lista pode ser compartilhada se T também pode
unsafe impl<T: Linked + Sync> Sync for IntrusiveList<T> {}

impl<T: Linked> IntrusiveList<T> {
    /// Cria uma lista vazia.
    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
        }
    }

    /// Número de elementos.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Verifica se está vazia.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Retorna referência ao primeiro elemento.
    pub fn front(&self) -> Option<&T> {
        if self.head.is_null() {
            None
        } else {
            unsafe { Some(&*self.head) }
        }
    }

    /// Retorna referência mutável ao primeiro elemento.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.head) }
        }
    }

    /// Retorna referência ao último elemento.
    pub fn back(&self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&*self.tail) }
        }
    }

    /// Retorna referência mutável ao último elemento.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.tail) }
        }
    }

    /// Adiciona elemento no final da lista.
    ///
    /// ## Safety:
    /// - O item não deve estar em outra lista
    /// - O item deve permanecer válido enquanto estiver na lista
    pub fn push_back(&mut self, item: &mut T) {
        debug_assert!(!item.is_linked(), "Item já está em uma lista!");

        let item_ptr = item as *mut T;
        item.set_next(ptr::null_mut());
        item.set_prev(self.tail);

        if self.tail.is_null() {
            // Lista estava vazia
            self.head = item_ptr;
        } else {
            // Conecta ao tail atual
            unsafe {
                (*self.tail).set_next(item_ptr);
            }
        }

        self.tail = item_ptr;
        self.len += 1;
    }

    /// Adiciona elemento no início da lista.
    pub fn push_front(&mut self, item: &mut T) {
        debug_assert!(!item.is_linked(), "Item já está em uma lista!");

        let item_ptr = item as *mut T;
        item.set_prev(ptr::null_mut());
        item.set_next(self.head);

        if self.head.is_null() {
            // Lista estava vazia
            self.tail = item_ptr;
        } else {
            // Conecta ao head atual
            unsafe {
                (*self.head).set_prev(item_ptr);
            }
        }

        self.head = item_ptr;
        self.len += 1;
    }

    /// Remove e retorna o primeiro elemento.
    pub fn pop_front(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            return None;
        }

        let item = unsafe { &mut *self.head };
        let next = item.next();

        if next.is_null() {
            // Era o único elemento
            self.head = ptr::null_mut();
            self.tail = ptr::null_mut();
        } else {
            // Atualiza o novo head
            unsafe {
                (*next).set_prev(ptr::null_mut());
            }
            self.head = next;
        }

        item.unlink();
        self.len -= 1;
        Some(item)
    }

    /// Remove e retorna o último elemento.
    pub fn pop_back(&mut self) -> Option<&mut T> {
        if self.tail.is_null() {
            return None;
        }

        let item = unsafe { &mut *self.tail };
        let prev = item.prev();

        if prev.is_null() {
            // Era o único elemento
            self.head = ptr::null_mut();
            self.tail = ptr::null_mut();
        } else {
            // Atualiza o novo tail
            unsafe {
                (*prev).set_next(ptr::null_mut());
            }
            self.tail = prev;
        }

        item.unlink();
        self.len -= 1;
        Some(item)
    }

    /// Remove um elemento específico da lista.
    ///
    /// ## Safety:
    /// - O item deve estar nesta lista (não em outra)
    pub fn remove(&mut self, item: &mut T) {
        if !item.is_linked() {
            return; // Não está em nenhuma lista
        }

        let prev = item.prev();
        let next = item.next();

        // Atualiza o anterior
        if prev.is_null() {
            // Era o head
            self.head = next;
        } else {
            unsafe {
                (*prev).set_next(next);
            }
        }

        // Atualiza o próximo
        if next.is_null() {
            // Era o tail
            self.tail = prev;
        } else {
            unsafe {
                (*next).set_prev(prev);
            }
        }

        item.unlink();
        self.len -= 1;
    }

    /// Limpa a lista, removendo todos os elementos.
    ///
    /// Nota: Não libera memória (os elementos são externos).
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    /// Verifica se um elemento está nesta lista.
    ///
    /// O(n) - percorre toda a lista.
    pub fn contains(&self, item: &T) -> bool {
        let item_ptr = item as *const T;
        let mut current = self.head;

        while !current.is_null() {
            if current == item_ptr as *mut T {
                return true;
            }
            current = unsafe { (*current).next() };
        }
        false
    }
}

impl<T: Linked> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro para implementar Linked automaticamente.
///
/// ## Exemplo:
/// ```rust
/// struct Task {
///     pid: u32,
///     run_next: *mut Task,
///     run_prev: *mut Task,
/// }
///
/// impl_linked!(Task, run_next, run_prev);
/// ```
#[macro_export]
macro_rules! impl_linked {
    ($type:ty, $next:ident, $prev:ident) => {
        impl $crate::klib::collections::intrusive::Linked for $type {
            fn next(&self) -> *mut Self {
                self.$next
            }
            fn prev(&self) -> *mut Self {
                self.$prev
            }
            fn set_next(&mut self, n: *mut Self) {
                self.$next = n;
            }
            fn set_prev(&mut self, p: *mut Self) {
                self.$prev = p;
            }
        }
    };
}
