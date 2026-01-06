//! # Configuração do RMM
//!
//! Este módulo contém todas as constantes e configurações do RMM.
//! Centraliza valores que afetam o comportamento do gerenciador de memória.
//!
//! ## Layout de Memória Virtual
//!
//! ```text
//! 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : Userspace (128 TB)
//! 0xFFFF_8000_0000_0000 - 0xFFFF_8FFF_FFFF_FFFF : HHDM (16 TB)
//! 0xFFFF_9000_0000_0000 - 0xFFFF_9FFF_FFFF_FFFF : Kernel Heap (1 TB)
//! 0xFFFF_C000_0000_0000 - 0xFFFF_CFFF_FFFF_FFFF : Driver Zone (1 TB)
//! 0xFFFF_D000_0000_0000 - 0xFFFF_D0FF_FFFF_FFFF : DMA Pool (256 GB)
//! 0xFFFF_E000_0000_0000 - 0xFFFF_E0FF_FFFF_FFFF : Kernel Stacks (256 GB)
//! 0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel Code
//! ```
//!
//! ## Zonas Físicas
//!
//! ```text
//! 0x0000_0000 - 0x00FF_FFFF : DMA Zone (0-16 MB)
//! 0x0100_0000 - 0xFFFF_FFFF : DMA32 Zone (16 MB - 4 GB)
//! 0x1_0000_0000+            : Normal Zone (> 4 GB)
//! ```

// =============================================================================
// TAMANHOS DE PÁGINA
// =============================================================================

/// Tamanho de uma página normal (4 KB)
pub const PAGE_SIZE: usize = 4096;

/// Shift para converter bytes em páginas
pub const PAGE_SHIFT: usize = 12;

/// Máscara para alinhar endereços a páginas
pub const PAGE_MASK: u64 = !(PAGE_SIZE as u64 - 1);

/// Tamanho de uma huge page (2 MB)
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Shift para huge pages
pub const HUGE_PAGE_SHIFT: usize = 21;

/// Tamanho de uma giant page (1 GB)
pub const GIANT_PAGE_SIZE: usize = 1024 * 1024 * 1024;

/// Shift para giant pages
pub const GIANT_PAGE_SHIFT: usize = 30;

// =============================================================================
// LAYOUT VIRTUAL
// =============================================================================

/// Base do Higher Half Direct Map
/// Toda RAM física é mapeada aqui: virt = HHDM_BASE + phys
pub const HHDM_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Tamanho máximo do HHDM (16 TB)
pub const HHDM_SIZE: u64 = 0x1000_0000_0000;

/// Base do Kernel Heap
pub const HEAP_BASE: u64 = 0xFFFF_9000_0000_0000;

/// Tamanho inicial do heap (16 MB)
pub const HEAP_INITIAL_SIZE: usize = 16 * 1024 * 1024;

/// Tamanho máximo do heap (1 TB)
pub const HEAP_MAX_SIZE: usize = 0x100_0000_0000;

/// Base da Driver Zone
pub const DRIVER_ZONE_BASE: u64 = 0xFFFF_C000_0000_0000;

/// Tamanho da Driver Zone (1 TB)
pub const DRIVER_ZONE_SIZE: u64 = 0x100_0000_0000;

/// Base do DMA Pool
pub const DMA_POOL_BASE: u64 = 0xFFFF_D000_0000_0000;

/// Tamanho do DMA Pool (256 GB)
pub const DMA_POOL_SIZE: u64 = 0x40_0000_0000;

/// Base dos Kernel Stacks
pub const KSTACK_BASE: u64 = 0xFFFF_E000_0000_0000;

/// Tamanho de cada kernel stack (32 KB)
pub const KSTACK_SIZE: usize = 32 * 1024;

/// Número máximo de kernel stacks
pub const KSTACK_MAX_COUNT: usize = 4096;

// =============================================================================
// ZONAS FÍSICAS
// =============================================================================

/// Fim da zona DMA (16 MB)
pub const ZONE_DMA_END: u64 = 16 * 1024 * 1024;

/// Fim da zona DMA32 (4 GB)
pub const ZONE_DMA32_END: u64 = 4 * 1024 * 1024 * 1024;

// =============================================================================
// CHUNK E CACHE
// =============================================================================

/// Tamanho de um chunk (2 MB = 512 frames)
///
/// CRÍTICO: Chunk deve ser >= maior alocação contígua comum (Huge Page = 2MB).
/// Com chunks menores, alocações de Huge Pages exigiriam travar múltiplos locks,
/// causando problemas de performance e risco de deadlock.
///
/// Trade-off:
/// - 2 MB: Suporta Huge Pages com 1 lock, bom para maioria dos casos
/// - 128 MB: Ideal para DMA buffers grandes, mas bitmap fica maior
///
/// Escolhemos 2 MB como trade-off entre granularidade e simplicidade.
pub const CHUNK_SIZE: usize = HUGE_PAGE_SIZE; // 2 MB

/// Número de frames por chunk (512 para 2 MB chunks)
pub const FRAMES_PER_CHUNK: usize = CHUNK_SIZE / PAGE_SIZE;

/// Tamanho do cache per-CPU (número de frames)
pub const PERCPU_CACHE_SIZE: usize = 32;

/// Batch size para refill do cache
pub const PERCPU_CACHE_BATCH: usize = 8;

// =============================================================================
// LOCK ORDERING
// =============================================================================

/// Regra de Lock Ordering para evitar deadlocks:
/// Sempre adquira locks de chunks em ordem ASCENDENTE de endereço físico.
///
/// Certo:  Lock(Chunk 1) -> Lock(Chunk 2)
/// Errado: Lock(Chunk 2) -> Lock(Chunk 1)
///
/// Para alocações que atravessam múltiplos chunks (raro, > 2MB),
/// adquira todos os locks necessários em ordem ascendente antes de modificar.
pub const LOCK_ORDER_ASCENDING: bool = true;

// =============================================================================
// SMP
// =============================================================================

/// Número máximo de CPUs suportadas
pub const MAX_CPUS: usize = 256; // TODO: Remover

/// Tamanho da cache line (para alinhamento)
pub const CACHE_LINE_SIZE: usize = 64;

// =============================================================================
// FLAGS DE PAGE TABLE (x86_64)
// =============================================================================

/// Página presente
pub const PTE_PRESENT: u64 = 1 << 0;

/// Página escrevível
pub const PTE_WRITABLE: u64 = 1 << 1;

/// Página acessível por userspace
pub const PTE_USER: u64 = 1 << 2;

/// Write-through caching
pub const PTE_WRITE_THROUGH: u64 = 1 << 3;

/// Cache desabilitado
pub const PTE_NO_CACHE: u64 = 1 << 4;

/// Página acessada
pub const PTE_ACCESSED: u64 = 1 << 5;

/// Página suja (escrita)
pub const PTE_DIRTY: u64 = 1 << 6;

/// Huge page (em PD ou PDPT)
pub const PTE_HUGE: u64 = 1 << 7;

/// Página global (não flush no CR3 switch)
pub const PTE_GLOBAL: u64 = 1 << 8;

/// No-Execute (requer NX bit habilitado)
pub const PTE_NO_EXEC: u64 = 1 << 63;

/// Máscara para extrair endereço físico de PTE
pub const PTE_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

// =============================================================================
// BUDDY ALLOCATOR
// =============================================================================

/// Ordem mínima do buddy (2^0 = 1 página = 4KB)
pub const BUDDY_MIN_ORDER: usize = 0;

/// Ordem máxima do buddy (2^10 = 1024 páginas = 4MB)
pub const BUDDY_MAX_ORDER: usize = 10;

/// Número de ordens no buddy system
pub const BUDDY_ORDERS: usize = BUDDY_MAX_ORDER + 1;

// =============================================================================
// SLAB ALLOCATOR
// =============================================================================

/// Tamanho mínimo de objeto no slab (8 bytes)
pub const SLAB_MIN_SIZE: usize = 8;

/// Tamanho máximo de objeto no slab (2048 bytes)
/// Acima disso, usa buddy diretamente
pub const SLAB_MAX_SIZE: usize = 2048;

/// Número de classes de tamanho no slab
/// 8, 16, 32, 64, 128, 256, 512, 1024, 2048 = 9 classes
pub const SLAB_SIZE_CLASSES: usize = 9;

/// Canary para detectar heap overflow
pub const SLAB_CANARY: u64 = 0xDEAD_BEEF_CAFE_BABE;

// =============================================================================
// DEBUG
// =============================================================================

/// Se true, zera frames ao alocar (mais lento, mais seguro)
pub const DEBUG_ZERO_ON_ALLOC: bool = cfg!(debug_assertions);

/// Se true, verifica invariantes após operações
pub const DEBUG_VERIFY_INVARIANTS: bool = cfg!(debug_assertions);

/// Se true, mantém estatísticas detalhadas
pub const DEBUG_DETAILED_STATS: bool = cfg!(debug_assertions);

// =============================================================================
// ASLR
// =============================================================================

/// Número de bits de entropia para ASLR do heap
pub const ASLR_ENTROPY_BITS: usize = 8;

/// Número de slots possíveis (2^8 = 256)
pub const ASLR_SLOTS: usize = 1 << ASLR_ENTROPY_BITS;

/// Alinhamento de cada slot ASLR (2 MB)
pub const ASLR_SLOT_ALIGN: usize = HUGE_PAGE_SIZE;
