/// Arquivo: klib/list/linked.rs
///
/// Propósito: Lista Duplamente Encadeada.
///
/// Detalhes de Implementação:
/// - Similar a std::collections::LinkedList.
/// - Usa Box<Node<T>>.

//! Doubly Linked List

use alloc::boxed::Box;
use core::ptr::NonNull;
use core::marker::PhantomData;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    element: T,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Self {
            next: None,
            prev: None,
            element,
        }
    }
}

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn push_back(&mut self, elt: T) {
        let mut node = Box::new(Node::new(elt));
        node.next = None;
        node.prev = self.tail;
        let node_ptr = NonNull::new(Box::into_raw(node));

        match self.tail {
            None => self.head = node_ptr,
            Some(tail) => unsafe {
                (*tail.as_ptr()).next = node_ptr;
            },
        }

        self.tail = node_ptr;
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                Some(head) => (*head.as_ptr()).prev = None,
            }

            self.len -= 1;
            node.element
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}
