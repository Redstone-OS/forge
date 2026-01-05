//! # Buddy Allocator
//!
//! Alocador de páginas em potências de 2.

use crate::rmm::addr::align_up;
use crate::rmm::config::{BUDDY_MAX_ORDER, BUDDY_MIN_ORDER, BUDDY_ORDERS, PAGE_SIZE};

/// Buddy allocator para blocos de páginas
pub struct BuddyAllocator {
    /// Free lists por ordem (order 0 = 1 página, order 10 = 1024 páginas)
    free_lists: [FreeList; BUDDY_ORDERS],
    /// Base do heap
    base: usize,
    /// Tamanho total
    size: usize,
}

/// Lista de blocos livres para uma ordem
struct FreeList {
    head: Option<*mut FreeBlock>,
    count: usize,
}

/// Bloco livre (header in-place)
struct FreeBlock {
    next: Option<*mut FreeBlock>,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        const EMPTY_LIST: FreeList = FreeList {
            head: None,
            count: 0,
        };
        Self {
            free_lists: [EMPTY_LIST; BUDDY_ORDERS],
            base: 0,
            size: 0,
        }
    }

    pub fn init(&mut self, base: usize, size: usize) {
        self.base = base;
        self.size = size;
        // TODO: Adicionar região à free list apropriada
    }

    /// Aloca bloco de ordem especificada
    pub fn alloc(&mut self, order: usize) -> Option<*mut u8> {
        if order > BUDDY_MAX_ORDER {
            return None;
        }

        // Procura ordem que tenha bloco disponível
        for o in order..=BUDDY_MAX_ORDER {
            if let Some(block) = self.free_lists[o].pop() {
                // Split se necessário
                for split_order in (order..o).rev() {
                    let buddy = unsafe { block.add(Self::order_size(split_order)) };
                    self.free_lists[split_order].push(buddy);
                }
                return Some(block);
            }
        }
        None
    }

    /// Libera bloco
    pub fn free(&mut self, ptr: *mut u8, order: usize) {
        // TODO: Implementar coalescing com buddy
        self.free_lists[order].push(ptr);
    }

    /// Calcula ordem necessária para um tamanho
    pub fn order_for_size(size: usize) -> usize {
        let pages = align_up(size, PAGE_SIZE) / PAGE_SIZE;
        pages.next_power_of_two().trailing_zeros() as usize
    }

    /// Tamanho de um bloco de ordem
    fn order_size(order: usize) -> usize {
        PAGE_SIZE << order
    }
}

impl FreeList {
    fn push(&mut self, ptr: *mut u8) {
        let block = ptr as *mut FreeBlock;
        unsafe {
            (*block).next = self.head;
        }
        self.head = Some(block);
        self.count += 1;
    }

    fn pop(&mut self) -> Option<*mut u8> {
        self.head.map(|block| {
            unsafe {
                self.head = (*block).next;
            }
            self.count -= 1;
            block as *mut u8
        })
    }
}
