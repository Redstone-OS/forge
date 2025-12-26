use crate::mm::alloc::buddy::BuddyAllocator;
use core::alloc::Layout;
use core::ptr::NonNull;
// use crate::mm::config::PAGE_SIZE;

/// Tamanho mínimo de bloco no Slab (16 bytes)
const MIN_BLOCK_SIZE: usize = 16;
/// Tamanho máximo de bloco gerenciado pelo Slab (2048 bytes)
/// Alocações maiores vão direto para o Buddy.
const MAX_BLOCK_SIZE: usize = 2048;

/// Bytes mágicos para detecção de Overflow (Canaries)
const CANARY_START: u64 = 0xDEAD_BEEF_CAFE_BABE;
const CANARY_END: u64 = 0xBAAD_F00D_DEAD_C0DE;
const CANARY_SIZE: usize = core::mem::size_of::<u64>();

/// Cabeçalho de um bloco livre dentro de um Slab
struct FreeObject {
    next: Option<NonNull<FreeObject>>,
}

/// Um Slab representa uma "página" (ou mais) dividida em objetos de tamanho fixo.
/// Simplificação: Por enquanto, o SlabAllocator mantém apenas uma lista encadeada global
/// de objetos livres para cada classe de tamanho (SC - Size Class).
struct SizeClass {
    block_size: usize,
    free_list: Option<NonNull<FreeObject>>,
}

impl SizeClass {
    const fn new(block_size: usize) -> Self {
        Self {
            block_size,
            free_list: None,
        }
    }

    unsafe fn push(&mut self, ptr: *mut u8) {
        let mut obj = NonNull::new_unchecked(ptr as *mut FreeObject);
        obj.as_mut().next = self.free_list;
        self.free_list = Some(obj);
    }

    unsafe fn pop(&mut self) -> Option<*mut u8> {
        if let Some(obj) = self.free_list {
            self.free_list = obj.as_ref().next;
            return Some(obj.as_ptr() as *mut u8);
        }
        None
    }
}

/// Alocador Slab para objetos pequenos
pub struct SlabAllocator {
    /// Classes de tamanho: 16, 32, 64, ..., 2048 (potências de 2)
    /// Índices:
    /// 0 -> 16
    /// 1 -> 32
    /// ...
    /// 7 -> 2048
    size_classes: [SizeClass; 8],
}

// Send seguro pois é protegido por Mutex externo
unsafe impl Send for SlabAllocator {}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            size_classes: [
                SizeClass::new(16),
                SizeClass::new(32),
                SizeClass::new(64),
                SizeClass::new(128),
                SizeClass::new(256),
                SizeClass::new(512),
                SizeClass::new(1024),
                SizeClass::new(2048),
            ],
        }
    }

    /// Aloca um objeto pequeno com proteção de Canary.
    ///
    /// Adiciona um cabeçalho e rodapé com "bytes mágicos".
    /// Se a free list estiver vazia, requisita memória do PAI (Buddy Allocator) e refila.
    ///
    /// # Layout em Memória
    /// `[ CANARY_START (8B) | PADDING (Align) | DADOS USUÁRIO | CANARY_END (8B) ]`
    pub unsafe fn alloc(&mut self, layout: Layout, buddy: &mut BuddyAllocator) -> *mut u8 {
        // Calcular tamanho total necessário incluindo Canaries e alinhamento
        let header_size = Self::align_up(CANARY_SIZE, layout.align());
        let footer_size = CANARY_SIZE;
        let payload_size = layout.size();

        let total_size = header_size + payload_size + footer_size;

        // Verificar se ainda cabe nos slabs (senão, repassa para Buddy direto?)
        // Nota: Se ficar grande demais, podemos simplesmente não usar canary ou deixar o HeapAllocator decidir.
        // Por simplicidade aqui, se estourar 2048, falhamos ou tentamos alocar do slab maior?
        // O HeapAllocator chama slab.alloc só se size <= 2048. Mas size AQUI cresceu.
        // Vamos assumir que HeapAllocator verifica o tamanho *original*.
        // Se `total_size` > 2048, allocamos do SlabIndex 7 (2048) ? Não vai caber.
        // ERRO DE DESIGN: Se o usuáiro pede 2048, +16 bytes = 2064. O SlabAllocator só tem slots de 2048.
        // SOLUÇÃO: O HeapAllocator deve considerar o overhead *antes* de decidir Slab vs Buddy.
        // MAS como o canary é implementação interna do Slab...
        // AJUSTE: O SlabAllocator vai tentar achar um slot que caiba `total_size`.
        // Se `total_size` > MAX_BLOCK_SIZE, retornamos null indicando que Slab não serve,
        // mas HeapAllocator já decidiu Slab...
        // FIX RÁPIDO: Se total_size > MAX_BLOCK_SIZE, delegamos pro Buddy (como fallback) ou panicamos?
        // Melhor delegar ao Buddy se ficar grande demais.
        if total_size > MAX_BLOCK_SIZE {
            // Caso especial: overflow do tamanho máximo do slab.
            // Para não quebrar, vamos alocar diretamente do Buddy "raw" (sem canary do slab, ou implementamos canary lá tbm).
            // Por consistência, vamos alocar do Buddy MAS retornar sem canary para esses limiares,
            // OU (melhor) implementamos canary manual on-top do buddy aqui.
            // Vamos simplificar: se estourar, aloca do buddy diretamente (sem proteção).
            // TODO: Mover lógica de canary para camada superior (HeapAllocator)?
            return buddy.alloc(layout);
        }

        let idx = self.index_for(total_size);

        // --- Início da lógica de alocação de bloco (Inner Alloc) ---
        let ptr = self.alloc_block(idx, buddy);
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        // --- Fim Inner Alloc ---

        // Escrever Canaries
        let user_ptr = ptr.add(header_size);

        // Header
        // let header_ptr = user_ptr.sub(CANARY_SIZE) as *mut u64; // Removido (unused)
        // ptr (início do bloco) -> ... -> user_ptr
        // Escrevemos CANARY_START logo antes de user_ptr? E se o padding for grande?
        // O Canary deve ficar FIXO em relação ao user_ptr ou ao bloco?
        // Melhor: Canary no início do BLOCO ALOCADO.
        // Mas user data precisa estar alinhado.

        // Reboot lógica de pointers:
        // Bloco: [ H | P | User | F ]
        // H = ptr (block start)
        // User = ptr + header_size (header_size ajustado para alinhar User)
        // F = User + payload_size

        // 1. Escrever Start Canary
        // Nota: ptr pode não estar alinhado para u64? O Slab garante alinhamento mínimo (ex: 16)?
        // MIN_BLOCK_SIZE=16, endereços de página 4096. Sim, estão alinhados a 8 pelo menos.
        (ptr as *mut u64).write(CANARY_START);

        // 2. Escrever End Canary
        let footer_ptr = user_ptr.add(payload_size) as *mut u64;
        // O footer pode estar desalinhado se payload_size não for múltiplo de 8.
        // write_unaligned é seguro aqui.
        footer_ptr.write_unaligned(CANARY_END);

        user_ptr
    }

    /// Helper interno p/ obter bloco bruto
    unsafe fn alloc_block(&mut self, idx: usize, buddy: &mut BuddyAllocator) -> *mut u8 {
        // Tenta alocar da free list existente
        if let Some(obj) = self.size_classes[idx].pop() {
            return obj;
        }

        // Free list vazia: aloca nova página do Buddy
        let page_layout = Layout::from_size_align_unchecked(4096, 4096);
        let page_ptr = buddy.alloc(page_layout);

        if page_ptr.is_null() {
            return core::ptr::null_mut(); // OOM no Buddy
        }

        // Dividir a página
        let block_size = self.size_classes[idx].block_size;
        let blocks = 4096 / block_size;

        let mut current_ptr = page_ptr;
        for _ in 0..blocks {
            self.size_classes[idx].push(current_ptr);
            current_ptr = current_ptr.add(block_size);
        }

        self.size_classes[idx]
            .pop()
            .unwrap_or(core::ptr::null_mut())
    }

    /// Alinha valor para cima
    fn align_up(val: usize, align: usize) -> usize {
        (val + align - 1) & !(align - 1)
    }

    /// Libera um objeto pequeno e verifica integridade (Canaries).
    ///
    /// # Argumentos
    /// - `ptr`: Ponteiro retornado por `alloc()`
    /// - `layout`: Layout original da alocação
    /// - `buddy`: Referência ao BuddyAllocator para liberar objetos oversized
    ///
    /// # Panics
    /// Panics se detectar corrupção de heap (canary inválido).
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout, buddy: &mut BuddyAllocator) {
        let header_size = Self::align_up(CANARY_SIZE, layout.align());
        let footer_size = CANARY_SIZE;
        let payload_size = layout.size();
        let total_size = header_size + payload_size + footer_size;

        if total_size > MAX_BLOCK_SIZE {
            // Foi alocado direto no Buddy (veja alloc quando total_size > MAX_BLOCK_SIZE)
            // O alloc delegou para buddy.alloc(layout) diretamente, sem canaries
            // Então aqui liberamos diretamente no buddy
            crate::ktrace!(
                "(Slab) Dealloc oversized -> Buddy ({} bytes)",
                layout.size()
            );
            buddy.dealloc(ptr, layout);
            return;
        }

        // Recuperar início do bloco real
        let block_ptr = ptr.sub(header_size);

        // 1. Checar Start Canary
        let start_canary = (block_ptr as *const u64).read();
        if start_canary != CANARY_START {
            crate::kerror!("(MM) CRITICAL: Heap Underflow detectado em {:p}!", ptr);
            crate::kerror!(
                "(MM) Esperado: {:#x}, Encontrado: {:#x}",
                CANARY_START,
                start_canary
            );
            panic!("HEAP CORRUPTION: Underflow");
        }

        // 2. Checar End Canary
        let footer_ptr = ptr.add(payload_size) as *const u64;
        let end_canary = footer_ptr.read_unaligned();
        if end_canary != CANARY_END {
            crate::kerror!("(MM) CRITICAL: Heap Overflow detectado em {:p}!", ptr);
            crate::kerror!(
                "(MM) Esperado: {:#x}, Encontrado: {:#x}",
                CANARY_END,
                end_canary
            );
            panic!("HEAP CORRUPTION: Overflow");
        }

        let idx = self.index_for(total_size);
        self.size_classes[idx].push(block_ptr);
    }

    /// Retorna o índice da classe de tamanho apropriada
    fn index_for(&self, size: usize) -> usize {
        if size <= 16 {
            return 0;
        }
        if size <= 32 {
            return 1;
        }
        if size <= 64 {
            return 2;
        }
        if size <= 128 {
            return 3;
        }
        if size <= 256 {
            return 4;
        }
        if size <= 512 {
            return 5;
        }
        if size <= 1024 {
            return 6;
        }
        if size <= 2048 {
            return 7;
        }
        panic!("SlabAllocator: tamanho {} muito grande", size);
    }
}
