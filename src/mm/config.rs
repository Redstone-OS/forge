//! # Configuração do Módulo de Memória
//!
//! Define constantes, feature flags e configurações globais do módulo MM.

// =============================================================================
// CONSTANTES DE TAMANHO
// =============================================================================

/// Tamanho de uma página (4 KiB)
pub const PAGE_SIZE: usize = 4096;

/// Tamanho de uma huge page (2 MiB)
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Máscara para alinhar endereços a página
pub const PAGE_MASK: usize = !(PAGE_SIZE - 1);

/// Bits de offset dentro de uma página
pub const PAGE_OFFSET_BITS: usize = 12;

// =============================================================================
// LAYOUT DE MEMÓRIA VIRTUAL
// =============================================================================

/// Base do Higher Half Direct Map (toda RAM física mapeada aqui)
/// Identity map de 0 a 4GB por agora, expandir para HHDM depois
pub const HHDM_BASE: usize = 0x0000_0000_0000_0000;

/// Limite do identity map atual (4GB)
pub const IDENTITY_MAP_LIMIT: usize = 0x1_0000_0000; // 4GB

/// Base do heap do kernel
pub const HEAP_VIRT_BASE: usize = 0xFFFF_9000_0000_0000;

/// Tamanho inicial do heap (16 MiB)
pub const HEAP_INITIAL_SIZE: usize = 16 * 1024 * 1024;

/// Endereço do scratch slot para zerar páginas
pub const SCRATCH_VIRT: usize = 0xFFFF_FE00_0000_0000;

// =============================================================================
// CONFIGURAÇÃO DO ALLOCATOR
// =============================================================================

/// Tamanho máximo para usar Slab (acima disso usa Buddy)
pub const SLAB_MAX_SIZE: usize = 2048;

/// Tamanhos de cache do Slab
pub const SLAB_SIZES: [usize; 8] = [16, 32, 64, 128, 256, 512, 1024, 2048];

/// Ordem máxima do Buddy allocator (2^9 = 512 páginas = 2MB)
pub const BUDDY_MAX_ORDER: usize = 10;

// =============================================================================
// CONFIGURAÇÃO SMP
// =============================================================================

/// Número máximo de CPUs suportadas
pub const MAX_CPUS: usize = 64;

/// Tamanho de linha de cache (para evitar false sharing)
pub const CACHE_LINE_SIZE: usize = 64;

// =============================================================================
// FLAGS DE PAGE TABLE
// =============================================================================

/// Presente
pub const PTE_PRESENT: u64 = 1 << 0;

/// Escrita permitida
pub const PTE_WRITABLE: u64 = 1 << 1;

/// Acessível em user mode
pub const PTE_USER: u64 = 1 << 2;

/// Write-through
pub const PTE_WRITE_THROUGH: u64 = 1 << 3;

/// Cache disabled
pub const PTE_CACHE_DISABLE: u64 = 1 << 4;

/// Acessada
pub const PTE_ACCESSED: u64 = 1 << 5;

/// Dirty
pub const PTE_DIRTY: u64 = 1 << 6;

/// Huge page (2MB em PD, 1GB em PDPT)
pub const PTE_HUGE: u64 = 1 << 7;

/// No Execute
pub const PTE_NO_EXECUTE: u64 = 1 << 63;

/// Máscara para extrair endereço físico de PTE
pub const PTE_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

// =============================================================================
// FUNÇÕES UTILITÁRIAS
// =============================================================================

/// Alinha valor para cima ao múltiplo de align
#[inline(always)]
pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

/// Alinha valor para baixo ao múltiplo de align
#[inline(always)]
pub const fn align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}

/// Verifica se valor está alinhado
#[inline(always)]
pub const fn is_aligned(val: usize, align: usize) -> bool {
    val & (align - 1) == 0
}
