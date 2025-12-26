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
// Retiramos PAGE_MASK antigo para evitar conflito com a máscara de PTE (u64).
// Se precisar de máscara de offset, use !PAGE_ADDR_MASK ou similar.

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

/// Endereço base do heap do kernel (Higher Half).
/// Coordenado com o VMM. Usado para alocações dinâmicas (Box, Vec).
pub const HEAP_VIRT_BASE: usize = 0xFFFF_9000_0000_0000;

/// Tamanho inicial do heap (16 MiB).
pub const HEAP_INITIAL_SIZE: usize = 16 * 1024 * 1024;

/// Endereço virtual fixo para o "Scratch Slot".
/// Usado para mapear temporariamente páginas físicas para zeragem/cópia.
/// Deve estar em uma região segura, não sobreposta pelo Identity Map ou Heap.
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
// FLAGS DE PAGE TABLE (x86_64)
// =============================================================================

/// Máscara para extrair endereço físico de uma PTE (bits 12-51)
pub const PAGE_MASK: u64 = 0x000F_FFFF_FFFF_F000;

/// Presente: A página está na memória física.
pub const PAGE_PRESENT: u64 = 1 << 0;

/// Writable: A escrita é permitida.
pub const PAGE_WRITABLE: u64 = 1 << 1;

/// User: Acessível em modo usuário (Ring 3).
pub const PAGE_USER: u64 = 1 << 2;

/// Write-through caching policy.
pub const PAGE_WRITE_THROUGH: u64 = 1 << 3;

/// Cache Disable: Desabilita cache para esta página.
pub const PAGE_CACHE_DISABLE: u64 = 1 << 4;

/// Accessed: O bit de acesso foi setado pela CPU.
pub const PAGE_ACCESSED: u64 = 1 << 5;

/// Dirty: A página foi escrita.
pub const PAGE_DIRTY: u64 = 1 << 6;

/// Huge Page: 2MB em PD, 1GB em PDPT.
pub const PAGE_HUGE: u64 = 1 << 7;

/// Global: A página não é invalidada no switch de CR3 (se PGE bit em CR4 estiver ativo).
pub const PAGE_GLOBAL: u64 = 1 << 8;

/// No Execute: A execução de código é proibida nesta página (bit 63).
pub const PAGE_NO_EXEC: u64 = 1 << 63;

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
