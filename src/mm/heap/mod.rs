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

use crate::mm::alloc::{BuddyAllocator, SlabAllocator};
use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};

/// Endereço virtual inicial do heap (Higher-Half)
/// Definido em `mm::config` para consistência. Alterações requerem ajustes no VMM/Bootloader.
pub const HEAP_START: usize = crate::mm::config::HEAP_VIRT_BASE;

/// Tamanho inicial do heap (16 MiB)
/// Definido em `mm::config`.
pub const HEAP_INITIAL_SIZE: usize = crate::mm::config::HEAP_INITIAL_SIZE;

/// Endereço real de início do Heap (pode variar com ASLR)
/// Inicializado em `init_heap`.
static HEAP_START_ADDR: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(crate::mm::config::HEAP_VIRT_BASE);

/// Retorna o endereço virtual inicial do Heap (runtime).
pub fn heap_start() -> usize {
    HEAP_START_ADDR.load(core::sync::atomic::Ordering::Relaxed)
}

/// Helper para ler Time Stamp Counter (TSC) para entropia
fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Orquestrador de Alocação de Memória (Composite: Slab + Buddy)
pub struct HeapAllocator {
    buddy: BuddyAllocator,
    slab: SlabAllocator,
    /// Rastreia o fim da região mapeada para permitir crescimento
    heap_end: usize,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            buddy: BuddyAllocator::new(),
            slab: SlabAllocator::new(),
            heap_end: 0,
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.buddy.init(start, size);
        self.heap_end = start + size;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // Objetos pequenos (< 2048) vão para Slab, grandes para Buddy
        if layout.size() <= 2048 {
            self.slab.alloc(layout, &mut self.buddy)
        } else {
            self.buddy.alloc(layout)
        }
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if layout.size() <= 2048 {
            // Slab precisa do buddy para liberar objetos oversized
            self.slab.dealloc(ptr, layout, &mut self.buddy)
        } else {
            self.buddy.dealloc(ptr, layout)
        }
    }

    // TODO: Implementar grow se necessário. Por enquanto, assumimos tamanho fixo inicial.
    // Para manter compatibilidade com a trait/interface anterior:
    pub unsafe fn grow(
        &mut self,
        _extra_size: usize,
        _pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
    ) -> bool {
        crate::kwarn!("(HeapAllocator) grow: Não implementado para Buddy/Slab ainda.");
        false
    }
}

/// Estrutura encapsulando o HeapAllocator protegido por Mutex
/// -----------------------------------------------------------
/// Garante exclusão mútua em cenários multicore simplificados.
/// Acesso ao allocator deve sempre passar pelo lock.
pub struct LockedHeap {
    inner: Mutex<HeapAllocator>,
}

impl LockedHeap {
    /// Construtor em tempo de compilação — sem heap inicializado
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(HeapAllocator::new()),
        }
    }

    /// Inicializa o heap com início e tamanho fornecidos
    /// -------------------------------------------------
    /// # Safety
    /// - Deve ser chamado apenas **uma vez** durante a inicialização do kernel
    /// - `heap_start` e `heap_size` devem estar mapeados em memória virtual válida
    pub unsafe fn init(&self, start: usize, size: usize) {
        crate::kdebug!("(Heap) init: início=", start);
        crate::kdebug!("(Heap) init: tamanho=", size as u64);
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
        crate::kdebug!("(Heap) grow: extra_size=", extra_size as u64);
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
        // DEBUG desativado - gera muito overhead em loops rápidos
        // static mut ALLOC_COUNT: usize = 0;
        // let count = ALLOC_COUNT;
        // ALLOC_COUNT += 1;

        // crate::ktrace!("(Heap) [H1] alloc entrada, count=", count as u64);
        // crate::ktrace!("(Heap) [H2] obtendo lock...");
        let mut guard = self.inner.lock();

        // crate::ktrace!("(Heap) [H3] lock OK, chamando alloc...");
        let ptr = guard.alloc(layout);

        // crate::ktrace!("(Heap) [H4] alloc retornou ptr=", ptr as u64);

        if ptr.is_null() {
            crate::kerror!("(Heap) OOM! size=", layout.size() as u64);
        }

        // Drop explícito do guard antes de retornar
        drop(guard);
        // crate::ktrace!("(Heap) [H5] guard dropped, retornando");

        ptr
    }

    /// Libera memória (apenas decrementa contador lógico)
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Sem log aqui - muito frequente
        self.inner.lock().dealloc(ptr, layout)
    }
}

/// Inicializa o heap do kernel
/// ---------------------------
/// - Mapeia todas as páginas virtuais correspondentes
/// - Inicializa o `ALLOCATOR` global
/// - Retorna `true` se sucesso, `false` se OOM ou falha de mapeamento
pub fn init(pmm: &mut crate::mm::pmm::BitmapFrameAllocator) -> bool {
    // Usando constantes importadas de config
    let base_addr = crate::mm::config::HEAP_VIRT_BASE;
    let heap_size = crate::mm::config::HEAP_INITIAL_SIZE;

    // --- ASLR (Heap Randomization) ---
    // Adiciona um offset aleatório ao endereço base para dificultar exploits.
    // IMPORTANTE: Limitamos a 64 slots (128 MiB max) para garantir que o heap
    // fique dentro da região PML4[288] pré-alocada pelo bootloader.
    // Offset máximo: 128 MiB (64 * 2MB)
    let tsc = rdtsc();
    // Usamos máscara 0x3F (64 slots) deslocada por 21 bits (2MB alignment)
    let random_offset = (tsc & 0x3F) as usize * 0x200000;

    let heap_start = base_addr + random_offset;

    // Atualiza a global atomic para que outros módulos (testes) saibam onde começa
    HEAP_START_ADDR.store(heap_start, core::sync::atomic::Ordering::Relaxed);

    crate::kinfo!("(Heap) Base=", base_addr);
    crate::kinfo!("(Heap) Offset ASLR=", random_offset as u64);
    crate::kinfo!("(Heap) Início=", heap_start);

    let pages = heap_size / 4096;
    if heap_size & (heap_size - 1) != 0 {
        crate::kerror!(
            "(Heap) FATAL: Tamanho do heap deve ser potencia de 2:",
            heap_size as u64
        );
        return false;
    }
    crate::kinfo!("(Heap) Mapeando páginas=", pages as u64);

    let flags = crate::mm::config::PAGE_PRESENT | crate::mm::config::PAGE_WRITABLE;

    let mut page_addr = heap_start;
    let heap_end = heap_start + heap_size;
    let mut pages_mapped = 0usize;

    crate::ktrace!("(Heap) Iniciando loop de mapeamento...");

    while page_addr < heap_end {
        if pages_mapped == 0 {
            crate::ktrace!("(Heap) Alocando primeiro frame...");
        }

        let frame = match pmm.allocate_frame() {
            Some(f) => f,
            None => {
                crate::kerror!("(Heap) init: OOM após páginas=", pages_mapped as u64);
                return false;
            }
        };

        if pages_mapped == 0 {
            crate::kdebug!("(Heap) Primeiro frame=", frame.addr());
        }

        // Importante: map_page_with_pmm está no módulo VMM
        if let Err(_e) = crate::mm::vmm::map_page_with_pmm(
            page_addr as u64,
            frame.addr(),
            crate::mm::vmm::MapFlags::from_bits_truncate(flags),
            pmm,
        ) {
            crate::kerror!("(Heap) init: mapeamento falhou em=", page_addr);
            return false;
        }

        pages_mapped += 1;
        page_addr += 4096;
    }
    crate::ktrace!("(Heap) páginas mapeadas OK=", pages_mapped as u64);

    unsafe {
        ALLOCATOR.init(heap_start, heap_size);
    }

    crate::kinfo!("(Heap) Inicializado");
    true
}

// =============================================================================
// IMPLEMENTAÇÕES MANUAIS DE ALOCAÇÃO (bypass __rust_alloc gerado)
// =============================================================================
// O compilador Rust pode gerar código para __rust_alloc que usa instruções SSE
// para memcpy/memset internos. Isso causa #UD se SSE não estiver configurado
// ou se o stack não estiver alinhado a 16 bytes.
//
// SOLUÇÃO: Fornecemos implementações manuais usando #[no_mangle] que
// delegam diretamente para nosso GlobalAlloc, evitando qualquer geração
// de código SSE pelo compilador.
//
// NOTA: O #[global_allocator] já define esses símbolos em Rust moderno,
// então usamos #[linkage = "weak"] não disponível. Como alternativa,
// confiamos que o GlobalAlloc seja usado corretamente pelo compilador
// e garantimos que SSE está habilitado ANTES de qualquer alocação.

// Helper para zeragem manual sem SSE
#[inline(always)]
unsafe fn manual_memset(ptr: *mut u8, val: u8, count: usize) {
    let mut i = 0;
    while i < count {
        core::ptr::write_volatile(ptr.add(i), val);
        i += 1;
    }
}

// Helper para cópia manual sem SSE
#[inline(always)]
unsafe fn manual_memcpy(dst: *mut u8, src: *const u8, count: usize) {
    let mut i = 0;
    while i < count {
        core::ptr::write_volatile(dst.add(i), core::ptr::read_volatile(src.add(i)));
        i += 1;
    }
}

/// Realloc manual que não depende de SSE
pub unsafe fn manual_realloc(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
    if let Ok(new_layout) = Layout::from_size_align(new_size, old_layout.align()) {
        let new_ptr = ALLOCATOR.alloc(new_layout);
        if !new_ptr.is_null() {
            let copy_size = core::cmp::min(old_layout.size(), new_size);
            manual_memcpy(new_ptr, ptr, copy_size);
            ALLOCATOR.dealloc(ptr, old_layout);
        }
        new_ptr
    } else {
        core::ptr::null_mut()
    }
}

/// Alloc zeroed manual que não depende de SSE
pub unsafe fn manual_alloc_zeroed(layout: Layout) -> *mut u8 {
    let ptr = ALLOCATOR.alloc(layout);
    if !ptr.is_null() {
        manual_memset(ptr, 0, layout.size());
    }
    ptr
}
