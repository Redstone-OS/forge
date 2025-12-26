use core::alloc::Layout;
use core::cmp::max;
use core::mem::size_of;
use core::ptr::NonNull;

use crate::mm::config::PAGE_SIZE;

/// Ordem máxima do Buddy System (0..=12)
/// Order 0 = 4 KiB
/// Order 12 = 4 KiB * 4096 = 16 MiB (Tamanho inicial total do heap)
const MAX_ORDER: usize = 12;

/// Cabeçalho de um bloco livre na lista encadeada
struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
}

impl FreeBlock {
    #[allow(dead_code)]
    fn new(next: Option<NonNull<FreeBlock>>) -> Self {
        Self { next }
    }
}

/// Alocador Buddy System para o Heap do Kernel
/// -------------------------------------------
/// Gerencia memória virtual em blocos de potência de 2.
/// - Minimiza fragmentação externa via coalescência.
/// - Ideal para alocações maiores que 2KB (abaixo disso, use Slab).
pub struct BuddyAllocator {
    /// Listas de blocos livres para cada ordem [0..MAX_ORDER]
    free_lists: [Option<NonNull<FreeBlock>>; MAX_ORDER + 1],
    /// Estatísticas
    allocated_bytes: usize,
    total_bytes: usize,
}

// O BuddyAllocator é protegido por um Mutex externo (LockedHeap), então é Send.
unsafe impl Send for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [None; MAX_ORDER + 1],
            allocated_bytes: 0,
            total_bytes: 0,
        }
    }

    /// Inicializa o alocador com uma região de memória contígua.
    ///
    /// # Safety
    /// - `heap_start` deve ser um ponteiro válido para memória mapeada e acessível.
    /// - `heap_size` deve ser potência de 2 e alinhado a PAGE_SIZE.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.total_bytes = heap_size;
        self.allocated_bytes = 0;

        // Adiciona a região inicial à free list da maior ordem possível
        self.add_to_free_list(heap_start, heap_size);
    }

    /// Adiciona um bloco de memória ao alocador (usado no init e grow)
    unsafe fn add_to_free_list(&mut self, addr: usize, size: usize) {
        let start = addr;
        let end = addr + size;
        let mut curr = start;

        // Decompor o tamanho em blocos de ordem apropriada
        while curr < end {
            let remaining = end - curr;
            let order = self.find_suitable_order(remaining);
            let block_size = 1 << (order + 12); // PAGE_SIZE << order

            // Inserir na lista
            self.push(curr, order);

            curr += block_size;
        }
    }

    /// Encontra a maior ordem que cabe em `size`
    fn find_suitable_order(&self, size: usize) -> usize {
        let pages = size / PAGE_SIZE;
        if pages == 0 {
            return 0;
        }

        let mut order = 0;
        while (1 << (order + 1)) <= pages && order < MAX_ORDER {
            order += 1;
        }
        order
    }

    /// Aloca memória
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = max(layout.size(), size_of::<FreeBlock>());
        let align = max(layout.align(), size_of::<FreeBlock>());
        let size = max(size, align); // Simplificação: tamanho >= alinhamento

        let pages_needed = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let target_order = self.size_to_order(pages_needed);

        // Tentar encontrar bloco na ordem alvo ou maior
        for order in target_order..=MAX_ORDER {
            if let Some(block_ptr) = self.pop(order) {
                // Bloco encontrado!
                let ptr = block_ptr.as_ptr() as usize;

                // Dividir (split) até chegar na ordem desejada
                for curr_order in (target_order..order).rev() {
                    let buddy_size = 1 << (curr_order + 12);
                    let buddy_addr = ptr + buddy_size;

                    // O bloco "superior" (buddy) vai para a free list da ordem menor
                    self.push(buddy_addr, curr_order);
                }

                self.allocated_bytes += 1 << (target_order + 12);
                return ptr as *mut u8;
            }
        }

        core::ptr::null_mut()
    }

    /// Libera memória
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(layout.size(), size_of::<FreeBlock>());
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let start_order = self.size_to_order(pages);

        self.allocated_bytes -= 1 << (start_order + 12);

        // Tentar fundir (coalesce) com buddies
        let mut curr_ptr = ptr as usize;
        let mut curr_order = start_order;

        while curr_order < MAX_ORDER {
            let block_size = 1 << (curr_order + 12);
            let buddy_addr = curr_ptr ^ block_size; // XOR encontra o endereço do buddy

            // Verificar se o buddy está na free list da mesma ordem
            if self.remove_from_list(buddy_addr, curr_order) {
                // Buddy encontrado e removido da free list! Fundir.
                // O endereço do bloco fundido é o menor entre os dois
                curr_ptr = core::cmp::min(curr_ptr, buddy_addr);
                curr_order += 1;
            } else {
                // Buddy não está livre ou tem outra ordem -> paramos de fundir
                break;
            }
        }

        // Devolve o bloco (potencialmente fundido) para a lista
        self.push(curr_ptr, curr_order);
    }

    /// Converte número de páginas em Ordem
    fn size_to_order(&self, pages: usize) -> usize {
        let mut order = 0;
        // 1 page = order 0
        // 2 pages = order 1
        // ...
        // 2^N pages = order N
        while (1 << order) < pages {
            order += 1;
        }
        order
    }

    /// Insere um bloco na free list de uma ordem
    unsafe fn push(&mut self, addr: usize, order: usize) {
        let mut block_ptr = NonNull::new_unchecked(addr as *mut FreeBlock);
        let next = self.free_lists[order];

        block_ptr.as_mut().next = next;
        self.free_lists[order] = Some(block_ptr);
    }

    /// Remove o primeiro bloco da free list de uma ordem
    unsafe fn pop(&mut self, order: usize) -> Option<NonNull<FreeBlock>> {
        if let Some(block_ptr) = self.free_lists[order] {
            self.free_lists[order] = block_ptr.as_ref().next;
            return Some(block_ptr);
        }
        None
    }

    /// Tenta remover um bloco específico da free list (usado no merge)
    /// Retorna true se encontrou e removeu.
    unsafe fn remove_from_list(&mut self, addr: usize, order: usize) -> bool {
        let mut curr = self.free_lists[order];
        let mut prev: Option<NonNull<FreeBlock>> = None;

        while let Some(node) = curr {
            if node.as_ptr() as usize == addr {
                // Encontramos!
                if let Some(mut p) = prev {
                    p.as_mut().next = node.as_ref().next;
                } else {
                    self.free_lists[order] = node.as_ref().next;
                }
                return true;
            }
            prev = Some(node);
            curr = node.as_ref().next;
        }
        false
    }
}
