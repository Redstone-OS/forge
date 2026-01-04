//! # Pool Centralizado de DMA (DMA Pool)
//!
//! Este módulo implementa um **alocador centralizado** para buffers de
//! DMA (Direct Memory Access). Dispositivos de hardware frequentemente
//! precisam de memória que:
//!
//! - Seja fisicamente contígua
//! - Tenha alinhamento específico
//! - Tenha endereço físico conhecido (para programar o dispositivo)
//!
//! ## Por que centralizado?
//! - **Controle**: A Base sabe quem alocou o quê
//! - **Segurança**: Impossível driver A usar DMA de driver B
//! - **Limpeza**: Fácil liberar tudo quando driver morre
//! - **IOMMU**: Preparado para proteção de DMA futura
//!
//! ## STUB:
//! Este módulo está parcialmente implementado. A alocação real de
//! memória física contígua depende do VMM do kernel.

use super::device::DeviceId;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES DO DMA POOL
// =============================================================================

/// Endereço virtual inicial do DMA Pool.
/// Separado da Driver Zone para facilitar mapeamento IOMMU futuro.
pub const DMA_POOL_START: u64 = 0xFFFF_D000_0000_0000;

/// Endereço virtual final do DMA Pool.
/// Tamanho: 64 GB (suficiente para muitos buffers DMA).
pub const DMA_POOL_END: u64 = 0xFFFF_D0FF_FFFF_FFFF;

/// Tamanho mínimo de um buffer DMA (uma página).
pub const MIN_DMA_SIZE: usize = 4096;

/// Tamanho máximo de um buffer DMA (256 MB).
/// Buffers maiores são raros e devem ser evitados.
pub const MAX_DMA_SIZE: usize = 256 * 1024 * 1024;

/// Alinhamento padrão para DMA (página).
pub const DMA_ALIGNMENT: usize = 4096;

// =============================================================================
// ESTRUTURA DE BUFFER DMA
// =============================================================================

/// Representa um buffer alocado para operações de DMA.
///
/// Contém tanto o endereço virtual (para CPU) quanto o endereço
/// físico (para programar o dispositivo de hardware).
#[derive(Debug, Clone)]
pub struct DmaBuffer {
    /// Endereço virtual (usado pela CPU para acessar os dados).
    pub virt: u64,

    /// Endereço físico (programado nos registradores do dispositivo).
    pub phys: u64,

    /// Tamanho do buffer em bytes.
    pub size: usize,

    /// ID do dispositivo que possui este buffer.
    pub owner: DeviceId,

    /// Flag indicando se o buffer está em uso.
    pub in_use: bool,

    /// Direção do DMA (para futuro suporte a coerência de cache).
    pub direction: DmaDirection,
}

/// Direção do fluxo de dados DMA.
///
/// Importante para coerência de cache e otimizações.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// Host → Device (CPU escreve, dispositivo lê).
    ToDevice,

    /// Device → Host (dispositivo escreve, CPU lê).
    FromDevice,

    /// Bidirecional (ambos leem e escrevem).
    Bidirectional,
}

// =============================================================================
// GERENCIADOR DO DMA POOL
// =============================================================================

/// Gerenciador centralizado de buffers DMA.
struct DmaPoolManager {
    /// Lista de todos os buffers alocados.
    buffers: Vec<DmaBuffer>,

    /// Próximo endereço virtual disponível.
    next_virt: u64,

    /// Total de bytes alocados.
    total_allocated: usize,

    /// Flag de inicialização.
    initialized: bool,
}

/// Instância global do gerenciador de DMA.
static DMA_POOL: Spinlock<DmaPoolManager> = Spinlock::new(DmaPoolManager {
    buffers: Vec::new(),
    next_virt: DMA_POOL_START,
    total_allocated: 0,
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o DMA Pool.
///
/// Configura as estruturas necessárias para alocação de buffers DMA.
pub fn init() {
    crate::kinfo!("(DMA) Inicializando DMA Pool...");

    let mut pool = DMA_POOL.lock();

    if pool.initialized {
        crate::kwarn!("(DMA) Pool já inicializado");
        return;
    }

    // TODO: Reservar região de memória física para DMA
    // A implementação real precisará:
    // 1. Alocar páginas físicas contíguas
    // 2. Mapear na região virtual DMA_POOL_START
    // 3. Configurar IOMMU se disponível

    pool.initialized = true;
    pool.next_virt = DMA_POOL_START;

    crate::kinfo!("(DMA) Pool pronto:");
    crate::kinfo!("  Início:", DMA_POOL_START);
    crate::kinfo!("  Fim:", DMA_POOL_END);
}

/// Aloca um buffer DMA.
///
/// ## Parâmetros:
/// - `size`: Tamanho desejado (será arredondado para cima)
/// - `owner`: ID do dispositivo alocador
/// - `direction`: Direção do fluxo de dados
///
/// ## Retorno:
/// Some(DmaBuffer) com ponteiros válidos, ou None em caso de falha.
///
/// ## STUB:
/// Atualmente não aloca memória física real. Retorna estrutura
/// simulada para permitir desenvolvimento de drivers.
pub fn alloc(size: usize, owner: DeviceId, direction: DmaDirection) -> Option<DmaBuffer> {
    // Validações
    if size == 0 || size > MAX_DMA_SIZE {
        crate::kerror!("(DMA) Tamanho inválido:", size);
        return None;
    }

    // Arredonda para múltiplo de página
    let aligned_size = align_up(size, DMA_ALIGNMENT);

    crate::kwarn!("(DMA) alloc() usando simulação! DMA real não implementado");

    let mut pool = DMA_POOL.lock();

    // Verifica espaço disponível
    if pool.next_virt + aligned_size as u64 > DMA_POOL_END {
        crate::kerror!("(DMA) Pool esgotado!");
        return None;
    }

    // Simula alocação
    let virt = pool.next_virt;
    let phys = virt - DMA_POOL_START + 0x1000_0000; // Offset fictício

    let buffer = DmaBuffer {
        virt,
        phys,
        size: aligned_size,
        owner,
        in_use: true,
        direction,
    };

    pool.buffers.push(buffer.clone());
    pool.next_virt += aligned_size as u64;
    pool.total_allocated += aligned_size;

    crate::kinfo!(
        "(DMA) Buffer alocado: virt=",
        virt,
        "phys=",
        phys,
        "size=",
        aligned_size
    );

    Some(buffer)
}

/// Libera um buffer DMA.
///
/// ## STUB:
/// Marca o buffer como livre, mas não libera memória real.
pub fn free(buffer: &DmaBuffer) {
    crate::kwarn!("(DMA) free() parcialmente implementado");

    let mut pool = DMA_POOL.lock();

    for buf in pool.buffers.iter_mut() {
        if buf.virt == buffer.virt && buf.owner == buffer.owner {
            buf.in_use = false;
            pool.total_allocated = pool.total_allocated.saturating_sub(buffer.size);
            crate::kinfo!("(DMA) Buffer liberado:", buffer.virt);
            return;
        }
    }

    crate::kwarn!("(DMA) Buffer não encontrado para liberação");
}

/// Libera todos os buffers DMA de um dispositivo.
///
/// Chamado quando driver é removido ou dispositivo isolado.
pub fn free_all_for_device(owner: DeviceId) {
    crate::kinfo!("(DMA) Liberando buffers do dispositivo:", owner.0);

    let mut pool = DMA_POOL.lock();
    let mut freed_count = 0;
    let mut freed_size = 0;

    for buf in pool.buffers.iter_mut() {
        if buf.owner == owner && buf.in_use {
            buf.in_use = false;
            freed_count += 1;
            freed_size += buf.size;
        }
    }

    pool.total_allocated = pool.total_allocated.saturating_sub(freed_size);

    crate::kinfo!(
        "(DMA) Liberados",
        freed_count,
        "buffers,",
        freed_size,
        "bytes"
    );
}

/// Retorna o endereço físico de um buffer DMA.
///
/// Usado para programar registradores de hardware.
pub fn phys_addr(buffer: &DmaBuffer) -> u64 {
    buffer.phys
}

/// Retorna o endereço virtual de um buffer DMA.
///
/// Usado pela CPU para acessar os dados.
pub fn virt_addr(buffer: &DmaBuffer) -> u64 {
    buffer.virt
}

/// Retorna ponteiro mutável para os dados do buffer.
///
/// ## Safety:
/// Caller deve garantir acesso exclusivo durante DMA.
pub unsafe fn as_mut_ptr(buffer: &DmaBuffer) -> *mut u8 {
    buffer.virt as *mut u8
}

/// Retorna estatísticas do DMA Pool.
pub fn get_stats() -> DmaStats {
    let pool = DMA_POOL.lock();

    DmaStats {
        total_allocated: pool.total_allocated,
        buffer_count: pool.buffers.iter().filter(|b| b.in_use).count(),
        pool_start: DMA_POOL_START,
        pool_end: DMA_POOL_END,
    }
}

/// Estatísticas do DMA Pool.
#[derive(Debug, Clone)]
pub struct DmaStats {
    /// Total de bytes alocados.
    pub total_allocated: usize,

    /// Número de buffers ativos.
    pub buffer_count: usize,

    /// Início do pool.
    pub pool_start: u64,

    /// Fim do pool.
    pub pool_end: u64,
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Arredonda valor para cima até múltiplo de align.
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Verifica se um endereço está no DMA Pool.
pub fn is_in_dma_pool(addr: u64) -> bool {
    addr >= DMA_POOL_START && addr < DMA_POOL_END
}
