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

    /// NOTA: Usa assembly para evitar SSE
    unsafe fn push(&mut self, ptr: *mut u8) {
        let obj_ptr = ptr as *mut usize; // Ponteiro para o campo 'next'
        let next_val = match self.free_list {
            Some(p) => p.as_ptr() as usize,
            None => 0, // null
        };

        // Escrever usando assembly
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) obj_ptr,
            in(reg) next_val,
            options(nostack, preserves_flags)
        );

        self.free_list = NonNull::new(ptr as *mut FreeObject);
    }

    /// NOTA: Usa assembly para evitar SSE na leitura
    unsafe fn pop(&mut self) -> Option<*mut u8> {
        if let Some(obj) = self.free_list {
            let obj_ptr = obj.as_ptr() as *const usize;

            // Ler o campo 'next' usando assembly
            let next_val: usize;
            core::arch::asm!(
                "mov {0}, [{1}]",
                out(reg) next_val,
                in(reg) obj_ptr,
                options(nostack, preserves_flags, readonly)
            );

            // Converter o valor lido para Option<NonNull<FreeObject>>
            if next_val == 0 {
                self.free_list = None;
            } else {
                self.free_list = NonNull::new(next_val as *mut FreeObject);
            }

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
        crate::ktrace!("(Slab) alloc: [S1] entrada");

        // Calcular tamanho total necessário incluindo Canaries e alinhamento
        let header_size = Self::align_up(CANARY_SIZE, layout.align());
        let footer_size = CANARY_SIZE;
        let payload_size = layout.size();

        let total_size = header_size + payload_size + footer_size;

        crate::ktrace!("(Slab) alloc: [S2] total_size=", total_size as u64);

        if total_size > MAX_BLOCK_SIZE {
            crate::ktrace!("(Slab) alloc: [S2a] -> buddy fallback");
            return buddy.alloc(layout);
        }

        crate::ktrace!("(Slab) alloc: [S3] index_for...");
        let idx = self.index_for(total_size);

        crate::ktrace!("(Slab) alloc: [S4] alloc_block idx=", idx as u64);

        // --- Início da lógica de alocação de bloco (Inner Alloc) ---
        let ptr = self.alloc_block(idx, buddy);
        if ptr.is_null() {
            crate::kerror!("(Slab) alloc: [S5] OOM!");
            return core::ptr::null_mut();
        }

        crate::ktrace!("(Slab) alloc: [S5] ptr=", ptr as u64);
        // --- Fim Inner Alloc ---

        // Escrever Canaries
        let user_ptr = ptr.add(header_size);

        // Bloco: [ H | P | User | F ]
        // H = ptr (block start)
        // User = ptr + header_size (header_size ajustado para alinhar User)
        // F = User + payload_size

        // 1. Escrever Start Canary usando assembly
        let canary_start_ptr = ptr as *mut u64;
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) canary_start_ptr,
            in(reg) CANARY_START,
            options(nostack, preserves_flags)
        );

        // 2. Escrever End Canary usando assembly
        let footer_ptr = user_ptr.add(payload_size) as *mut u64;
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) footer_ptr,
            in(reg) CANARY_END,
            options(nostack, preserves_flags)
        );

        user_ptr
    }

    /// Helper interno p/ obter bloco bruto
    ///
    /// NOTA: Usa while loops para evitar SSE
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

        // Dividir a página usando while
        let block_size = self.size_classes[idx].block_size;
        let blocks = 4096 / block_size;

        let mut current_ptr = page_ptr;
        let mut i = 0usize;
        while i < blocks {
            self.size_classes[idx].push(current_ptr);
            current_ptr = current_ptr.add(block_size);
            i += 1;
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
                "(Slab) Dealloc oversized -> Buddy. Size=",
                layout.size() as u64
            );
            buddy.dealloc(ptr, layout);
            return;
        }

        // Recuperar início do bloco real
        let block_ptr = ptr.sub(header_size);

        // 1. Checar Start Canary
        let start_canary = (block_ptr as *const u64).read();
        if start_canary != CANARY_START {
            crate::kerror!("(MM) CRITICAL: Heap Underflow detectado em=", ptr as u64);
            crate::kerror!("(MM) Encontrado=", start_canary);
            panic!("HEAP CORRUPTION: Underflow");
        }

        // 2. Checar End Canary
        let footer_ptr = ptr.add(payload_size) as *const u64;
        let end_canary = footer_ptr.read_unaligned();
        if end_canary != CANARY_END {
            crate::kerror!("(MM) CRITICAL: Heap Overflow detectado em=", ptr as u64);
            crate::kerror!("(MM) Encontrado=", end_canary);
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
