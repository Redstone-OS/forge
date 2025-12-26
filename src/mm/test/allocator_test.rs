//! Testes de Alocadores (Buddy + Slab)
//!
//! Este módulo testa os alocadores de memória usando o heap real do kernel,
//! não uma arena estática. Isso garante que os testes validem o comportamento
//! real do sistema de memória.

use crate::mm::alloc::{BuddyAllocator, SlabAllocator};
use alloc::vec::Vec;
use core::alloc::Layout;

/// Valida que o heap está corretamente mapeado antes de rodar testes.
/// Retorna true se o heap está acessível.
fn validate_heap_mapping() -> bool {
    use crate::mm::vmm::translate_addr;
    let heap_start = crate::mm::heap::heap_start() as u64;

    // Verificar se o início do heap está mapeado
    if translate_addr(heap_start).is_none() {
        crate::kerror!("(AllocTest) Heap não mapeado em {:#x}!", heap_start);
        return false;
    }

    // Verificar se o fim do heap também está mapeado
    let heap_end = heap_start + crate::mm::heap::HEAP_INITIAL_SIZE as u64 - 4096;
    if translate_addr(heap_end).is_none() {
        crate::kerror!("(AllocTest) Fim do heap não mapeado em {:#x}!", heap_end);
        return false;
    }

    crate::kinfo!(
        "(AllocTest) Heap validado: {:#x} - {:#x}",
        heap_start,
        heap_end
    );
    true
}

/// Arena de teste alocada dinamicamente no heap.
/// Usamos Vec<u8> para garantir que a memória está corretamente mapeada.
struct DynamicTestArena {
    buffer: Vec<u8>,
}

impl DynamicTestArena {
    /// Aloca uma arena de teste de 1 MiB no heap.
    fn new() -> Self {
        let size = 1024 * 1024; // 1 MiB
        let mut buffer = Vec::with_capacity(size);
        // Preencher com zeros para garantir que as páginas estão mapeadas
        buffer.resize(size, 0);
        Self { buffer }
    }

    fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    fn size(&self) -> usize {
        self.buffer.len()
    }
}

pub fn run_alloc_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE ALOCADORES            ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    // CRÍTICO: Validar heap antes de rodar qualquer teste
    if !validate_heap_mapping() {
        crate::kerror!("(AllocTest) Heap não validado! Abortando testes de alocador.");
        return;
    }

    test_buddy_basic();
    test_slab_basic();
    test_slab_canary_integrity();
    test_integration();

    crate::kinfo!("(AllocTest) ✓ Todos os testes de alocador passaram.");
}

fn test_buddy_basic() {
    crate::kinfo!("(Test) BuddyAllocator: Basic Alloc/Dealloc...");

    // Alocar arena dinamicamente no heap
    let arena = DynamicTestArena::new();
    let mut buddy = BuddyAllocator::new();

    unsafe {
        let start = arena.as_ptr() as usize;
        let size = arena.size();
        buddy.init(start, size);

        // Alloc 4KB (Order 0)
        let layout1 = Layout::from_size_align(4096, 4096).unwrap();
        let ptr1 = buddy.alloc(layout1);
        if ptr1.is_null() {
            panic!("(Buddy) Falha alloc 4KB");
        }
        crate::ktrace!("(Buddy) Alloc 4KB -> {:p}", ptr1);

        // Alloc 8KB (Order 1)
        let layout2 = Layout::from_size_align(8192, 4096).unwrap();
        let ptr2 = buddy.alloc(layout2);
        if ptr2.is_null() {
            panic!("(Buddy) Falha alloc 8KB");
        }
        crate::ktrace!("(Buddy) Alloc 8KB -> {:p}", ptr2);

        // Dealloc em ordem inversa
        buddy.dealloc(ptr2, layout2);
        buddy.dealloc(ptr1, layout1);

        // Tentar alocar tudo de novo (deve conseguir se o merge funcionou)
        let ptr_all = buddy.alloc(Layout::from_size_align(size, 4096).unwrap());
        if ptr_all.is_null() {
            panic!("(Buddy) Falha ao realocar tudo (fragmentação/merge falhou?)");
        }
        crate::ktrace!("(Buddy) Realloc Full 1MB -> {:p}", ptr_all);

        buddy.dealloc(ptr_all, Layout::from_size_align(size, 4096).unwrap());
    }
    crate::kinfo!("(Test) BuddyAllocator: OK");
}

fn test_slab_basic() {
    crate::kinfo!("(Test) SlabAllocator: Basic Alloc/Dealloc...");

    let arena = DynamicTestArena::new();
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = arena.as_ptr() as usize;
        let size = arena.size();
        buddy.init(start, size);

        // Alloc 32 bytes
        let layout32 = Layout::from_size_align(32, 16).unwrap();
        let ptr1 = slab.alloc(layout32, &mut buddy);
        if ptr1.is_null() {
            panic!("(Slab) Falha alloc 32B");
        }

        // Escrever padrão
        core::ptr::write_volatile(ptr1 as *mut u64, 0xCAFEBABE);

        // Alloc 32 bytes de novo
        let ptr2 = slab.alloc(layout32, &mut buddy);
        if ptr2.is_null() {
            panic!("(Slab) Falha alloc 32B (2)");
        }

        if ptr1 == ptr2 {
            panic!("(Slab) Retornou mesmo ponteiro!");
        }

        crate::ktrace!("(Slab) Ptr1: {:p}, Ptr2: {:p}", ptr1, ptr2);

        slab.dealloc(ptr1, layout32, &mut buddy);
        slab.dealloc(ptr2, layout32, &mut buddy);
    }
    crate::kinfo!("(Test) SlabAllocator: OK");
}

fn test_slab_canary_integrity() {
    crate::kinfo!("(Test) SlabAllocator: Canary Integrity...");

    let arena = DynamicTestArena::new();
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = arena.as_ptr() as usize;
        let size = arena.size();
        buddy.init(start, size);

        // Alloc 16 bytes
        let layout = Layout::from_size_align(16, 16).unwrap();
        let ptr = slab.alloc(layout, &mut buddy);
        assert!(!ptr.is_null());

        // Header: align_up(8, 16) = 16 bytes de overhead
        let header_overhead = 16;
        let block_ptr = ptr.sub(header_overhead);
        let canary_start = (block_ptr as *const u64).read();

        if canary_start != 0xDEAD_BEEF_CAFE_BABE {
            crate::kerror!(
                "(Test) Start Canary AUSENTE ou incorreto: {:#x}",
                canary_start
            );
            panic!("Test Canary Integrity Failed");
        }

        // Footer: user_ptr + size(16)
        let footer_ptr = ptr.add(16) as *const u64;
        let canary_end = footer_ptr.read_unaligned();

        if canary_end != 0xBAAD_F00D_DEAD_C0DE {
            crate::kerror!("(Test) End Canary AUSENTE ou incorreto: {:#x}", canary_end);
            panic!("Test Canary Integrity Failed");
        }

        slab.dealloc(ptr, layout, &mut buddy);
    }
    crate::kinfo!("(Test) Canary Integrity: OK");
}

#[allow(dead_code)]
fn test_slab_overflow() {
    crate::kinfo!("(Test) SlabAllocator: OVERFLOW TEST (Deve Panic)...");

    let arena = DynamicTestArena::new();
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = arena.as_ptr() as usize;
        let size = arena.size();
        buddy.init(start, size);

        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = slab.alloc(layout, &mut buddy);

        // Corromper Footer
        crate::kwarn!("(Test) Corrompendo heap intencionalmente...");
        let footer_ptr = ptr.add(32) as *mut u64;
        footer_ptr.write(0xDEAD_DEAD_DEAD_DEAD);

        // Isso deve triggar panic
        slab.dealloc(ptr, layout, &mut buddy);
    }
}

fn test_integration() {
    crate::kinfo!("(Test) Integration: HeapAllocator Logic...");

    let arena = DynamicTestArena::new();
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = arena.as_ptr() as usize;
        let size = arena.size();
        buddy.init(start, size);

        // Caso Slab
        let ptr_small = slab.alloc(Layout::from_size_align(128, 16).unwrap(), &mut buddy);
        assert!(!ptr_small.is_null());
        slab.dealloc(
            ptr_small,
            Layout::from_size_align(128, 16).unwrap(),
            &mut buddy,
        );

        // Caso Buddy (4096)
        let ptr_large = buddy.alloc(Layout::from_size_align(4096, 4096).unwrap());
        assert!(!ptr_large.is_null());
        buddy.dealloc(ptr_large, Layout::from_size_align(4096, 4096).unwrap());
    }
    crate::kinfo!("(Test) Integration: OK");
}
