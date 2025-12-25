//! Kernel Heap Allocator (Bump) — Documentação Profissional e Industrial
//! ===============================================================
//!
//! Este módulo implementa um **Global Allocator** simples baseado em *bump allocator*,
//! protegido por `Mutex`. É projetado para o uso **early-kernel** do Redstone OS,
//! fornecendo alocações dinâmicas determinísticas e previsíveis durante a inicialização.
//!
//! # Filosofia de Design
//! - **Simplicidade:** design minimalista para bootstrap do kernel.
//! - **Determinístico:** aloca sempre subindo o ponteiro `next`, sem listas de free complexas.
//! - **Segurança:** valida OOM e retorna `null_mut` quando não há memória suficiente.
//! - **Progresso:** permite crescimento dinâmico do heap via mapeamento de novas páginas.
//!
//! # Limitações
//! - Não é um allocator de produção completo: **não recicla fragmentos**.
//! - `dealloc` apenas decrementa contador lógico; memória só é reutilizada quando `allocations == 0`.
//! - Não há proteção multithread além do `Mutex`; para múltiplos núcleos, atomic ops seriam necessárias.
//!
//! # Contratos importantes
//! - `init_heap(...)` **deve** ser chamado **uma vez** após mapear páginas físicas para o heap.
//! - `HEAP_START` deve estar **sincronizado com o VMM**, pois mapeia memória virtual.
//! - Todos os acessos `unsafe` são restritos ao mapeamento de memória e inicialização.
//!
//! # Melhorias possíveis
//! - Implementar crescimento automático transparente (já parcialmente suportado via `grow`).
//! - Substituir bump allocator por um allocator com free list ou slab para produção.
//! - Adicionar suporte a múltiplos núcleos e alocações atomizadas.

use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};

/// Endereço virtual inicial do heap (Higher-Half)
/// -----------------------------------------------
/// Deve ser consistente com o VMM. Alterações requerem ajustes no mapeamento.
pub const HEAP_START: usize = 0xFFFF_9000_0000_0000;

/// Tamanho inicial do heap (4 MiB)
/// ---------------------------------
/// Este valor define o heap "bootstrap". Pode crescer via `grow`.
pub const HEAP_INITIAL_SIZE: usize = 4 * 1024 * 1024;

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
        self.inner.lock().init(start, size);
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
        self.inner.lock().grow(extra_size, pmm)
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    /// Aloca memória no heap
    /// ---------------------
    /// Retorna `null_mut` em caso de OOM.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.lock().alloc(layout);
        if ptr.is_null() {
            crate::kerror!("Heap OOM: falha ao alocar {} bytes", layout.size());
        }
        ptr
    }

    /// Libera memória (apenas decrementa contador lógico)
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
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
                    crate::kerror!(
                        "Heap grow: sem frames suficientes para {} bytes extras",
                        extra_size
                    );
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
    let page_range = HEAP_START..(HEAP_START + HEAP_INITIAL_SIZE);

    for page_addr in (page_range).step_by(crate::mm::pmm::FRAME_SIZE) {
        let frame = match pmm.allocate_frame() {
            Some(f) => f,
            None => {
                crate::kerror!("Heap init: sem frames suficientes");
                return false;
            }
        };

        let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;
        if unsafe { !crate::mm::vmm::map_page_with_pmm(page_addr as u64, frame.addr, flags, pmm) } {
            crate::kerror!("Heap init: falha ao mapear página {:#x}", page_addr);
            return false;
        }
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_INITIAL_SIZE);
    }

    crate::kinfo!(
        "Heap inicializado: {} KiB em {:#x}",
        HEAP_INITIAL_SIZE / 1024,
        HEAP_START
    );
    true
}
