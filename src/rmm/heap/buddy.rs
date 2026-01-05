//! # Buddy Allocator
//!
//! Allocator para alocações grandes (> 2KB) usando buddy system.
//!
//! ## Buddy System
//!
//! O buddy system divide memória em blocos de potência de 2.
//! Quando um bloco é liberado, tenta fazer merge com seu "buddy" (vizinho).
//!
//! ```text
//! Ordem 0: 4KB   (2^0 páginas = 1 página)
//! Ordem 1: 8KB   (2^1 páginas = 2 páginas)
//! Ordem 2: 16KB  (2^2 páginas = 4 páginas)
//! ...
//! Ordem 10: 4MB  (2^10 páginas = 1024 páginas)
//! ```
//!
//! ## Estrutura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     BuddyAllocator                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  free_lists[11]  (uma lista por ordem)                      │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐      ┌─────────┐      │
//! │  │ Ordem 0 │ │ Ordem 1 │ │ Ordem 2 │ ...  │ Ordem 10│      │
//! │  │   4KB   │ │   8KB   │ │   16KB  │      │   4MB   │      │
//! │  │  ┌───┐  │ │  ┌───┐  │ │  ┌───┐  │      │  ┌───┐  │      │
//! │  │  │ ● │  │ │  │ ● │  │ │  │ ● │  │      │  │nil│  │      │
//! │  │  └─│─┘  │ │  └─│─┘  │ │  └─│─┘  │      │  └───┘  │      │
//! │  │    ↓    │ │    ↓    │ │    ↓    │      │         │      │
//! │  │  block  │ │  block  │ │  block  │      │         │      │
//! │  └─────────┘ └─────────┘ └─────────┘      └─────────┘      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Algoritmo
//!
//! **Alocação:**
//! 1. Calcula ordem mínima necessária
//! 2. Procura bloco livre dessa ordem
//! 3. Se não há, procura bloco maior e faz split
//!
//! **Liberação:**
//! 1. Marca bloco como livre
//! 2. Calcula endereço do buddy
//! 3. Se buddy está livre, faz merge e repete

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::*;

use core::alloc::Layout;
use core::ptr::NonNull;

/// Número de ordens (0 a 10)
const ORDER_COUNT: usize = BUDDY_ORDERS;

/// Tamanho mínimo do bloco (ordem 0)
const MIN_BLOCK_SIZE: usize = PAGE_SIZE; // 4KB

/// Tamanho máximo do bloco (ordem 10)
const MAX_BLOCK_SIZE: usize = PAGE_SIZE << BUDDY_MAX_ORDER; // 4MB

/// Nó de lista livre
///
/// Armazenado no próprio bloco livre (intrusive list).
#[repr(C)]
struct FreeBlock {
    next: *mut FreeBlock,
    prev: *mut FreeBlock,
}

impl FreeBlock {
    fn new() -> Self {
        Self {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }
}

/// Buddy Allocator
pub struct BuddyAllocator {
    /// Base do heap
    base: VirtAddr,
    /// Tamanho total
    size: usize,
    /// Listas livres por ordem
    free_lists: [*mut FreeBlock; ORDER_COUNT],
    /// Contador de blocos livres por ordem
    free_counts: [usize; ORDER_COUNT],
    /// Bitmap para rastrear estado de split (1 bit por par de buddies)
    /// Se bit = 1, os buddies foram splitados
    split_bitmap: [u64; 256], // Suporta até 16KB blocos
}

impl BuddyAllocator {
    /// Cria novo buddy allocator
    pub fn new(base: VirtAddr, size: usize) -> Self {
        let mut allocator = Self {
            base,
            size,
            free_lists: [core::ptr::null_mut(); ORDER_COUNT],
            free_counts: [0; ORDER_COUNT],
            split_bitmap: [0; 256],
        };

        // Inicializa com blocos da maior ordem possível
        allocator.init_free_blocks();

        allocator
    }

    /// Inicializa blocos livres
    fn init_free_blocks(&mut self) {
        let max_order = self.max_order_for_size(self.size);
        let block_size = MIN_BLOCK_SIZE << max_order;
        let num_blocks = self.size / block_size;

        for i in 0..num_blocks {
            let addr = self.base.as_u64() + (i * block_size) as u64;
            self.add_to_free_list(addr as *mut FreeBlock, max_order);
        }
    }

    /// Aloca memória
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size().max(layout.align()).max(MIN_BLOCK_SIZE);
        let order = self.order_for_size(size);

        if order >= ORDER_COUNT {
            return None; // Muito grande
        }

        // Procura bloco livre
        let block = self.alloc_order(order)?;

        NonNull::new(block as *mut u8)
    }

    /// Aloca bloco de ordem específica
    fn alloc_order(&mut self, order: usize) -> Option<*mut u8> {
        // Procura bloco livre dessa ordem ou maior
        for o in order..ORDER_COUNT {
            if !self.free_lists[o].is_null() {
                // Remove da free list
                let block = self.remove_from_free_list(o);

                // Se ordem maior, faz split
                if o > order {
                    self.split_to_order(block, o, order);
                }

                return Some(block as *mut u8);
            }
        }

        None
    }

    /// Libera memória
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size().max(layout.align()).max(MIN_BLOCK_SIZE);
        let order = self.order_for_size(size);
        let block = ptr.as_ptr() as *mut FreeBlock;

        self.free_order(block, order);
    }

    /// Libera bloco de ordem específica, tentando merge
    fn free_order(&mut self, block: *mut FreeBlock, order: usize) {
        if order >= ORDER_COUNT - 1 {
            // Ordem máxima, não pode fazer merge
            self.add_to_free_list(block, order);
            return;
        }

        // Calcula endereço do buddy
        let buddy_addr = self.buddy_addr(block as u64, order);
        let buddy = buddy_addr as *mut FreeBlock;

        // Verifica se buddy está livre
        if self.is_free(buddy, order) {
            // Remove buddy da free list
            self.remove_block_from_free_list(buddy, order);

            // Merge: bloco com menor endereço é o novo bloco maior
            let merged = if (block as u64) < buddy_addr {
                block
            } else {
                buddy
            };

            // Recursivamente tenta merge na ordem superior
            self.free_order(merged, order + 1);
        } else {
            // Não pode fazer merge, adiciona à free list
            self.add_to_free_list(block, order);
        }
    }

    /// Split de bloco maior para ordem menor
    fn split_to_order(&mut self, block: *mut FreeBlock, from_order: usize, to_order: usize) {
        let mut current_order = from_order;
        let mut current_block = block as u64;

        while current_order > to_order {
            current_order -= 1;

            // Calcula endereço do buddy (metade do bloco)
            let block_size = MIN_BLOCK_SIZE << current_order;
            let buddy_addr = current_block + block_size as u64;

            // Adiciona buddy à free list
            self.add_to_free_list(buddy_addr as *mut FreeBlock, current_order);

            // Marca como splitado
            self.mark_split(current_block, current_order, true);
        }
    }

    /// Calcula endereço do buddy
    fn buddy_addr(&self, addr: u64, order: usize) -> u64 {
        let block_size = (MIN_BLOCK_SIZE << order) as u64;
        let relative = addr - self.base.as_u64();
        let buddy_relative = relative ^ block_size;
        self.base.as_u64() + buddy_relative
    }

    /// Calcula ordem necessária para tamanho
    fn order_for_size(&self, size: usize) -> usize {
        let mut order = 0;
        let mut block_size = MIN_BLOCK_SIZE;

        while block_size < size && order < ORDER_COUNT {
            order += 1;
            block_size <<= 1;
        }

        order
    }

    /// Ordem máxima que cabe no tamanho
    fn max_order_for_size(&self, size: usize) -> usize {
        let mut order = ORDER_COUNT - 1;

        while order > 0 {
            let block_size = MIN_BLOCK_SIZE << order;
            if block_size <= size {
                return order;
            }
            order -= 1;
        }

        0
    }

    // -------------------------------------------------------------------------
    // Free List Management
    // -------------------------------------------------------------------------

    /// Adiciona bloco à free list
    fn add_to_free_list(&mut self, block: *mut FreeBlock, order: usize) {
        unsafe {
            (*block).next = self.free_lists[order];
            (*block).prev = core::ptr::null_mut();

            if !self.free_lists[order].is_null() {
                (*self.free_lists[order]).prev = block;
            }

            self.free_lists[order] = block;
        }

        self.free_counts[order] += 1;
    }

    /// Remove primeiro bloco da free list
    fn remove_from_free_list(&mut self, order: usize) -> *mut FreeBlock {
        let block = self.free_lists[order];

        if block.is_null() {
            return block;
        }

        unsafe {
            self.free_lists[order] = (*block).next;

            if !self.free_lists[order].is_null() {
                (*self.free_lists[order]).prev = core::ptr::null_mut();
            }
        }

        self.free_counts[order] -= 1;
        block
    }

    /// Remove bloco específico da free list
    fn remove_block_from_free_list(&mut self, block: *mut FreeBlock, order: usize) {
        unsafe {
            let prev = (*block).prev;
            let next = (*block).next;

            if !prev.is_null() {
                (*prev).next = next;
            } else {
                self.free_lists[order] = next;
            }

            if !next.is_null() {
                (*next).prev = prev;
            }
        }

        self.free_counts[order] -= 1;
    }

    /// Verifica se bloco está na free list
    fn is_free(&self, block: *mut FreeBlock, order: usize) -> bool {
        let mut current = self.free_lists[order];

        while !current.is_null() {
            if current == block {
                return true;
            }
            unsafe {
                current = (*current).next;
            }
        }

        false
    }

    // -------------------------------------------------------------------------
    // Split Bitmap
    // -------------------------------------------------------------------------

    /// Marca bloco como splitado ou não
    fn mark_split(&mut self, addr: u64, order: usize, split: bool) {
        let index = self.split_bitmap_index(addr, order);
        let word = index / 64;
        let bit = index % 64;

        if word < self.split_bitmap.len() {
            if split {
                self.split_bitmap[word] |= 1 << bit;
            } else {
                self.split_bitmap[word] &= !(1 << bit);
            }
        }
    }

    /// Verifica se bloco está splitado
    fn is_split(&self, addr: u64, order: usize) -> bool {
        let index = self.split_bitmap_index(addr, order);
        let word = index / 64;
        let bit = index % 64;

        if word < self.split_bitmap.len() {
            (self.split_bitmap[word] & (1 << bit)) != 0
        } else {
            false
        }
    }

    /// Calcula índice no bitmap de split
    fn split_bitmap_index(&self, addr: u64, order: usize) -> usize {
        let block_size = MIN_BLOCK_SIZE << order;
        let relative = (addr - self.base.as_u64()) as usize;
        (relative / block_size) + (order * 1024) // Offset por ordem
    }

    // -------------------------------------------------------------------------
    // Extensão
    // -------------------------------------------------------------------------

    /// Estende o allocator
    pub fn extend(&mut self, additional: usize) {
        let old_size = self.size;
        self.size += additional;

        // Adiciona novos blocos
        let max_order = self.max_order_for_size(additional);
        let block_size = MIN_BLOCK_SIZE << max_order;
        let num_blocks = additional / block_size;

        for i in 0..num_blocks {
            let addr = self.base.as_u64() + old_size as u64 + (i * block_size) as u64;
            self.add_to_free_list(addr as *mut FreeBlock, max_order);
        }
    }

    // -------------------------------------------------------------------------
    // Estatísticas
    // -------------------------------------------------------------------------

    /// Total de blocos livres
    pub fn total_free_blocks(&self) -> usize {
        self.free_counts.iter().sum()
    }

    /// Memória livre total
    pub fn free_bytes(&self) -> usize {
        let mut total = 0;
        for (order, &count) in self.free_counts.iter().enumerate() {
            total += count * (MIN_BLOCK_SIZE << order);
        }
        total
    }

    /// Dump da free list para debug
    pub fn dump(&self) {
        crate::kinfo!("=== Buddy Allocator ===");
        crate::kinfo!(
            "  Base: 0x{:x}, Size: {} KB",
            self.base.as_u64(),
            self.size / 1024
        );

        for (order, &count) in self.free_counts.iter().enumerate() {
            if count > 0 {
                let block_size = MIN_BLOCK_SIZE << order;
                crate::kinfo!(
                    "  Order {}: {} blocks x {} KB = {} KB free",
                    order,
                    count,
                    block_size / 1024,
                    (count * block_size) / 1024
                );
            }
        }
    }
}

// Safety: BuddyAllocator é protegido por lock externo
unsafe impl Send for BuddyAllocator {}
