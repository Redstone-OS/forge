//! Kernel Heap Allocator (Bump) — Documentação Profissional e Segura
//! ---------------------------------------------------------------
//! Implementa `GlobalAlloc` com um *bump allocator* simples protegido por `Mutex`.
//! Projetado para uso no early-kernel do Redstone OS: rápido, determinístico e
//! suficiente para inicialização e estruturas estáticas (Box, Vec, String).
//!
//! ### Filosofia
//! - Simplicidade e previsibilidade: design minimalista para bootstrap.
//! - Não é um allocator de produção: não recycles fragmentos (vazamento intencional),
//!   mas fornece comportamento aceitável enquanto o kernel inicializa subsistemas mais
//!   avançados (slab/linked freelist/smalloc etc).
//!
//! ### Contratos importantes
//! - `init_heap(...)` **DEVE** ser chamado exatamente uma vez antes da primeira
//!   alocação dinâmica. Chamadas repetidas não são suportadas.
//! - `init_heap` mapeia páginas virtuais em `HEAP_START..HEAP_START+HEAP_SIZE` usando
//!   o PMM passado. Por isso ela recebe `&mut BitmapFrameAllocator` diretamente para
//!   evitar deadlocks de locks aninhados.
//! - O endereço `HEAP_START` é o *higher-half* acordado com o VMM; altere junto ao VMM.
//!
//! ### Segurança / Limitações
//! - `GlobalAlloc` requer `unsafe` — a implementação faz acesso a ponteiros brutos.
//! - `BumpAllocator::dealloc` não reutiliza memória; apenas decrementa um contador
//!   lógico. O reset do `next` só acontece quando `allocations == 0` para reduzir
//!   vazamento em cenários de boot-test. Não confie nisso para liberar memória em
//!   cargas de trabalho longas.
//! - `align_up` lida com `align == 0` e evita UB.
//!
//! ### Melhorias possíveis
//! - Substituir por um allocator que recicle páginas (bitmap + free list).
//! - Implementar proteção para múltiplos núcleos: usar atomic ops para `allocations`.
//! - Suportar expansão do heap (grow) mapeando mais frames dinamicamente.
//!

use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};

/// Endereço virtual inicial do heap (higher-half).
/// Mantenha sincronizado com o VMM.
pub const HEAP_START: usize = 0xFFFF_9000_0000_0000;

/// Tamanho do heap (1 MiB por padrão). Pode ser aumentado no futuro.
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

// Global allocator exposto ao resto do kernel (Box/Vec/String).
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Encapsula um BumpAllocator protegido por Mutex.
pub struct LockedHeap {
    inner: Mutex<BumpAllocator>,
}

impl LockedHeap {
    /// Construção em tempo de compilação — sem heap ainda.
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(BumpAllocator::new()),
        }
    }

    /// Inicializa o allocator com início e tamanho. `unsafe` porque faz mutação global.
    ///
    /// Deve ser chamada **após** mapear as páginas virtuais do heap (ou dentro da
    /// função que mapeia e inicializa em sequência).
    pub unsafe fn init(&self, start: usize, size: usize) {
        self.inner.lock().init(start, size);
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}

/// Bump Allocator simples:
/// - Aloca sempre subindo `next`.
/// - Não tem free list real; `dealloc` apenas ajusta um contador lógico.
/// - Ideal para early-boot; substitua quando precisar de reciclagem.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Inicializa limites do heap.
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start.saturating_add(heap_size);
        self.next = heap_start;
        self.allocations = 0;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // Validar align (align == 0 não faz sentido; tratar como 1)
        let align = if layout.align() == 0 {
            1
        } else {
            layout.align()
        };

        let alloc_start = align_up(self.next, align);
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        if alloc_end > self.heap_end || alloc_start < self.heap_start {
            // OOM
            core::ptr::null_mut()
        } else {
            self.next = alloc_end;
            self.allocations = self.allocations.saturating_add(1);
            alloc_start as *mut u8
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // Evitar underflow: use saturating_sub para robustez em caso de chamadas erradas.
        if self.allocations > 0 {
            self.allocations = self.allocations.saturating_sub(1);
        }

        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    if align <= 1 {
        return addr;
    }
    // align é potência de dois no uso típico; contudo, esta formula funciona para qualquer valor >0.
    (addr + align - 1) & !(align - 1)
}

/// Inicializa o Heap.
///
/// # Mudança Importante
///
/// Esta função agora recebe o PMM diretamente como parâmetro ao invés de
/// receber uma closure de mapeamento. Isso é necessário porque:
///
/// 1. O VMM precisa alocar frames para criar page tables
/// 2. Se o VMM tentar adquirir o lock do PMM enquanto o heap já tem o lock,
///    ocorre DEADLOCK
/// 3. Passando o PMM diretamente, evitamos a necessidade de lock duplo
///
/// # Pré-requisitos
/// - PMM deve estar inicializado
/// - VMM deve estar inicializado (scratch slot pronto)
///
/// # Safety
/// Deve ser chamada apenas uma vez durante a inicialização do kernel
pub fn init_heap(pmm: &mut crate::mm::pmm::BitmapFrameAllocator) {
    let page_range = HEAP_START..(HEAP_START + HEAP_SIZE);

    for page_addr in (page_range).step_by(crate::mm::pmm::FRAME_SIZE) {
        // Alocar frame físico para esta página do heap
        let frame = pmm.allocate_frame().expect("No frames for heap");

        let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;

        // Usar map_page_with_pmm para evitar deadlock
        // (já temos o lock do PMM, então passamos ele diretamente)
        let success =
            unsafe { crate::mm::vmm::map_page_with_pmm(page_addr as u64, frame.addr, flags, pmm) };

        if !success {
            panic!("Heap: Falha ao mapear página {:#x}", page_addr);
        }
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }

    crate::kinfo!("Heap: {} KiB em {:#x}", HEAP_SIZE / 1024, HEAP_START);
}
