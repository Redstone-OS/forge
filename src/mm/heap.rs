//! # Kernel Heap Allocator
//!
//! O `heap` fornece alocação dinâmica de memória (`Box`, `Vec`, `String`) para o kernel.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Dynamic Allocation:** Permite que o kernel use estruturas de dados que não têm tamanho conhecido em tempo de compilação.
//! - **Global Allocator:** Implementa a trait `GlobalAlloc` do Rust, integrando-se nativamente com a `alloc` crate.
//!
//! ## 🏗️ Arquitetura Atual: Bump Allocator (Temporário)
//! Atualmente, o kernel utiliza um **Bump Allocator** (também conhecido como Arena Allocator):
//! - **Pointer Bump:** `alloc` simplesmente retorna o ponteiro atual e incrementa o offset.
//! - **No Free:** `dealloc` é (quase) uma operação vazia. A memória nunca é reutilizada, a menos que *tudo* seja liberado.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Velocidade de Boot:** Alocação é O(1) (puro incremento de ponteiro). Zero overhead de busca de blocos livres.
//! - **Determinismo:** O layout de memória durante o boot é estritamente sequencial e previsível.
//! - **Simplicidade:** Implementação trivial de auditar, sem metadados complexos (headers/footers) que poderiam ser corrompidos.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica CRÍTICA)
//! - **Memory Leak by Design:** Como `dealloc` não recicla memória, qualquer driver ou serviço que aloque/desaloque repetidamente vai exaurir a RAM rapidamente.
//! - **Fragmentação:** Não há coalescência de blocos.
//! - **Single Global Lock:** Assim como no PMM, o `LockedHeap` usa um `Mutex` global, serializando todas as alocações do kernel.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical)** Migrar para **Slab Allocator** (objetos pequenos fixos) + **Buddy System** (páginas).
//!   - *Meta:* Permitir `cargo build` e uso normal de coleções sem vazar memória.
//! - [ ] **TODO: (Scalability)** Implementar **Per-CPU Caches** (sem lock) para alocações pequenas.
//!   - *Motivo:* Reduzir contenção do lock global do heap em workloads intensivos.
//! - [ ] **TODO: (Security)** Adicionar **Canaries/Guard Bytes** ao redor de alocações.
//!   - *Risco:* Detectar Heap Overflow antes que corrompa dados vizinhos.
//! - [ ] **TODO: (Safety)** Implementar **Randomization (ASLR-like)** para o heap base.

use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};

/// Endereço virtual inicial do heap (Higher-Half)
/// -----------------------------------------------
/// Deve ser consistente com o VMM. Alterações requerem ajustes no mapeamento.
pub const HEAP_START: usize = 0xFFFF_9000_0000_0000;

/// Tamanho inicial do heap (16 MiB)
/// ---------------------------------
/// Aumentado para suportar múltiplos serviços no initramfs.
pub const HEAP_INITIAL_SIZE: usize = 16 * 1024 * 1024;

/// Global allocator exposto a todo o kernel (Box, Vec, String)
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Estrutura encapsulando o BumpAllocator protegido por Mutex
/// -----------------------------------------------------------
/// Garante exclusão mútua em cenários multicore simplificados.
/// Acesso ao allocator deve sempre passar pelo lock.
pub struct LockedHeap {
    inner: Mutex<BumpAllocator>,
}

impl LockedHeap {
    /// Construtor em tempo de compilação — sem heap inicializado
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(BumpAllocator::new()),
        }
    }

    /// Inicializa o heap com início e tamanho fornecidos
    /// -------------------------------------------------
    /// # Safety
    /// - Deve ser chamado apenas **uma vez** durante a inicialização do kernel
    /// - `heap_start` e `heap_size` devem estar mapeados em memória virtual válida
    pub unsafe fn init(&self, start: usize, size: usize) {
        crate::kdebug!("(Heap) init: start={:#x} size={}", start, size);
        self.inner.lock().init(start, size);
        crate::kdebug!("(Heap) init: OK");
    }

    /// Cresce o heap dinamicamente adicionando `extra_size` bytes
    /// ----------------------------------------------------------
    /// Aloca novas páginas físicas usando PMM e mapeia no VMM.
    /// Retorna `true` se o crescimento foi bem-sucedido.
    pub unsafe fn grow(
        &self,
        extra_size: usize,
        pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
    ) -> bool {
        crate::kdebug!("(Heap) grow: extra_size={}", extra_size);
        let result = self.inner.lock().grow(extra_size, pmm);
        if result {
            crate::kdebug!("(Heap) grow: OK");
        } else {
            crate::kerror!("(Heap) grow: FALHOU!");
        }
        result
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    /// Aloca memória no heap
    /// ---------------------
    /// Retorna `null_mut` em caso de OOM.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.lock().alloc(layout);

        if ptr.is_null() {
            // SEMPRE logar OOM - é crítico
            crate::kerror!(
                "(Heap) OOM! size={} align={}",
                layout.size(),
                layout.align()
            );
        }

        ptr
    }

    /// Libera memória (apenas decrementa contador lógico)
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Sem log aqui - muito frequente
        self.inner.lock().dealloc(ptr, layout)
    }
}

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
        for page in (self.heap_end..new_end).step_by(crate::mm::pmm::FRAME_SIZE) {
            match pmm.allocate_frame() {
                Some(frame) => {
                    let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;
                    unsafe {
                        crate::mm::vmm::map_page_with_pmm(page as u64, frame.addr, flags, pmm);
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

/// Inicializa o heap do kernel
/// ---------------------------
/// - Mapeia todas as páginas virtuais correspondentes
/// - Inicializa o `ALLOCATOR` global
/// - Retorna `true` se sucesso, `false` se OOM ou falha de mapeamento
pub fn init_heap(pmm: &mut crate::mm::pmm::BitmapFrameAllocator) -> bool {
    let pages = HEAP_INITIAL_SIZE / 4096;
    crate::kinfo!(
        "(Heap) Mapeando {} páginas ({} KiB)...",
        pages,
        HEAP_INITIAL_SIZE / 1024
    );

    let page_range = HEAP_START..(HEAP_START + HEAP_INITIAL_SIZE);
    let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;

    for page_addr in (page_range).step_by(4096) {
        let frame = match pmm.allocate_frame() {
            Some(f) => f,
            None => {
                crate::kerror!("(Heap) init: OOM");
                return false;
            }
        };

        if unsafe { !crate::mm::vmm::map_page_with_pmm(page_addr as u64, frame.addr, flags, pmm) } {
            crate::kerror!("(Heap) init: mapeamento falhou");
            return false;
        }
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_INITIAL_SIZE);
    }

    crate::kinfo!("(Heap) Inicializado: {} KiB", HEAP_INITIAL_SIZE / 1024);
    true
}
