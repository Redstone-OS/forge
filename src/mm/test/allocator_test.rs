use crate::mm::alloc::{BuddyAllocator, SlabAllocator};
use core::alloc::Layout;

// Arena estática para testes isolados (1 MiB para garantir espaço para várias ordens)
// Alinhada a 4096 para satisfazer requisitos do Buddy
#[repr(align(4096))]
struct TestArena([u8; 1024 * 1024]);

static mut TEST_ARENA: TestArena = TestArena([0; 1024 * 1024]);

pub fn run_alloc_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE ALOCADORES            ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_buddy_basic();
    test_slab_basic();
    test_slab_canary_integrity(); // Verifica se canaries estão sendo escritos
    test_integration(); // Buddy + Slab interagindo

    // test_slab_overflow(); // Descomente para testar o Panic (vai travar o boot)

    crate::kinfo!("(AllocTest) ✓ Todos os testes de alocador passaram.");
}

fn test_buddy_basic() {
    crate::kinfo!("(Test) BuddyAllocator: Basic Alloc/Dealloc...");
    let mut buddy = BuddyAllocator::new();

    unsafe {
        let start = TEST_ARENA.0.as_ptr() as usize;
        let size = 1024 * 1024; // 1 MiB
        buddy.init(start, size);

        // Alloc 4KB (Order 0)
        let layout1 = Layout::from_size_align(4096, 4096).unwrap();
        let ptr1 = buddy.alloc(layout1);
        if ptr1.is_null() {
            panic!("(Buddy) Falha alloc 4KB");
        }
        crate::ktrace!("(Buddy) Alloc 4KB -> {:p}", ptr1);

        // Alloc 8KB (Order 1) - deve ser vizinho ou próximo
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
        // 1 MiB inteiro
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
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = TEST_ARENA.0.as_ptr() as usize;
        let size = 1024 * 1024;
        buddy.init(start, size);

        // Alloc 32 bytes
        let layout32 = Layout::from_size_align(32, 16).unwrap();
        let ptr1 = slab.alloc(layout32, &mut buddy);
        if ptr1.is_null() {
            panic!("(Slab) Falha alloc 32B");
        }

        // Escrever padrão
        core::ptr::write_volatile(ptr1 as *mut u64, 0xCAFEBABE);

        // Alloc 32 bytes de novo (deve vir do mesmo slab/bloco)
        let ptr2 = slab.alloc(layout32, &mut buddy);
        if ptr2.is_null() {
            panic!("(Slab) Falha alloc 32B (2)");
        }

        if ptr1 == ptr2 {
            panic!("(Slab) Retornou mesmo ponteiro!");
        }

        crate::ktrace!("(Slab) Ptr1: {:p}, Ptr2: {:p}", ptr1, ptr2);

        slab.dealloc(ptr1, layout32);
        slab.dealloc(ptr2, layout32);
    }
    crate::kinfo!("(Test) SlabAllocator: OK");
}

fn test_slab_canary_integrity() {
    crate::kinfo!("(Test) SlabAllocator: Canary Integrity...");
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = TEST_ARENA.0.as_ptr() as usize;
        let size = 1024 * 1024;
        buddy.init(start, size);

        // Alloc 16 bytes
        let layout = Layout::from_size_align(16, 16).unwrap();
        let ptr = slab.alloc(layout, &mut buddy);
        assert!(!ptr.is_null());

        // Verificar se os bytes mágicos estão lá manualmente

        // Header (Tamanho do canary é 8, align 16 -> Header align_up(8, 16) = 16 bytes)
        // Se CANARY_SIZE=8 e align=16: align_up(8, 16) -> 16.
        // Então ptr está em block_ptr + 16.
        // O canary (8 bytes) foi escrito em block_ptr + 0?
        // Verifique a impl de slab: (ptr as *mut u64).write(CANARY_START) onde ptr é block_ptr.
        // Sim, o canary está no início do bloco.

        let header_overhead = 16; // align_up(8, 16)
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

        slab.dealloc(ptr, layout);
    }
    crate::kinfo!("(Test) Canary Integrity: OK");
}

#[allow(dead_code)]
fn test_slab_overflow() {
    crate::kinfo!("(Test) SlabAllocator: OVERFLOW TEST (Deve Panic)...");
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = TEST_ARENA.0.as_ptr() as usize;
        let size = 1024 * 1024;
        buddy.init(start, size);

        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = slab.alloc(layout, &mut buddy);

        // Corromper Footer
        crate::kwarn!("(Test) Corrompendo heap intencionalmente...");
        let footer_ptr = ptr.add(32) as *mut u64;
        footer_ptr.write(0xDEAD_DEAD_DEAD_DEAD);

        // Isso deve triggar panic
        slab.dealloc(ptr, layout);
    }
}

fn test_integration() {
    crate::kinfo!("(Test) Integration: HeapAllocator Logic...");
    // Aqui testamos a lógica de seleção (size <= 2048 -> Slab, > -> Buddy)
    // Simulamos manualmente
    let mut buddy = BuddyAllocator::new();
    let mut slab = SlabAllocator::new();

    unsafe {
        let start = TEST_ARENA.0.as_ptr() as usize;
        let size = 1024 * 1024;
        buddy.init(start, size);

        // Caso Slab
        let ptr_small = slab.alloc(Layout::from_size_align(128, 16).unwrap(), &mut buddy);
        assert!(!ptr_small.is_null());
        slab.dealloc(ptr_small, Layout::from_size_align(128, 16).unwrap());

        // Caso Buddy (4096)
        let ptr_large = buddy.alloc(Layout::from_size_align(4096, 4096).unwrap());
        assert!(!ptr_large.is_null());
        buddy.dealloc(ptr_large, Layout::from_size_align(4096, 4096).unwrap());
    }
    crate::kinfo!("(Test) Integration: OK");
}
