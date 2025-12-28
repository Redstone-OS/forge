use core::alloc::Layout;

/// Implementação do Bump Allocator
/// --------------------------------
/// Aloca sempre subindo `next`. Não possui free list real.
/// Ideal para early-boot; substitua em produção por allocator completo.
pub struct BumpAllocator {
    heap_start: usize,  // Início do heap virtual
    heap_end: usize,    // Fim do heap virtual
    next: usize,        // Próxima posição livre
    allocations: usize, // Contador de alocações ativas
}

impl BumpAllocator {
    /// Construtor em tempo de compilação
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Inicializa limites do heap
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
        self.allocations = 0;
    }

    /// Aloca memória com layout específico
    /// ------------------------------------
    /// Retorna ponteiro nulo se não houver espaço suficiente.
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let align = if layout.align() == 0 {
            1
        } else {
            layout.align()
        };
        let alloc_start = align_up(self.next, align);
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(), // overflow detectado
        };

        if alloc_end > self.heap_end {
            return core::ptr::null_mut(); // sem espaço disponível
        }

        self.next = alloc_end;
        self.allocations = self.allocations.saturating_add(1);
        alloc_start as *mut u8
    }

    /// Libera memória (apenas decrementa contador)
    /// --------------------------------------------
    /// O reset do ponteiro `next` acontece apenas quando `allocations == 0`.
    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        if self.allocations > 0 {
            self.allocations = self.allocations.saturating_sub(1);
        }
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }

    /// Cresce o heap mapeando novas páginas físicas
    /// -------------------------------------------
    /// Necessita de PMM para alocar frames físicos.
    /// Retorna `false` se não houver frames suficientes.
    pub fn grow(
        &mut self,
        extra_size: usize,
        pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
    ) -> bool {
        let new_end = self.heap_end + extra_size;
        for page in (self.heap_end..new_end).step_by(crate::mm::config::PAGE_SIZE) {
            match pmm.allocate_frame() {
                Some(frame) => {
                    let flags = crate::mm::config::PAGE_PRESENT | crate::mm::config::PAGE_WRITABLE;
                    if let Err(e) = crate::mm::vmm::map_page_with_pmm(
                        page as u64,
                        frame.addr(),
                        crate::mm::vmm::MapFlags::from_bits_truncate(flags),
                        pmm,
                    ) {
                        crate::kerror!("(Heap) grow: falha de mapeamento: {}", e);
                        return false;
                    }
                }
                None => {
                    crate::kerror!("(Heap) grow: sem frames para {} bytes extras", extra_size);
                    return false;
                }
            }
        }
        self.heap_end = new_end;
        true
    }
}

/// Alinha o endereço `addr` para cima de acordo com `align`
/// ----------------------------------------------------------
/// Suporta qualquer `align > 0`. Para alocações típicas, `align` é potência de 2.
fn align_up(addr: usize, align: usize) -> usize {
    if align <= 1 {
        return addr;
    }
    (addr + align - 1) & !(align - 1)
}
