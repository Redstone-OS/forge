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
//! ## Arquitetura
//!
//! O DMA Pool aloca frames físicos do PMM e mapeia-os na região virtual
//! DMA_POOL_START. A conversão virtual→física é O(1) usando:
//!
//! ```text
//! phys = virt - DMA_POOL_START + first_frame_phys
//! ```
//!
//! ## Por que centralizado?
//! - **Controle**: A Base sabe quem alocou o quê
//! - **Segurança**: Impossível driver A usar DMA de driver B
//! - **Limpeza**: Fácil liberar tudo quando driver morre
//! - **IOMMU**: Preparado para proteção de DMA futura

use super::device::DeviceId;
// TODO: Migrar para RMM APIs quando disponíveis
// A API antiga do PMM foi removida. Este módulo precisa ser refatorado
// para usar crate::rmm::driver::dma::* quando estiver pronto.
use crate::rmm::addr::PhysAddr;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// Stubs temporários até migração completa
struct MapFlags;
impl MapFlags {
    const WRITABLE: Self = Self;
    const NO_EXECUTE: Self = Self;
    const NO_CACHE: Self = Self;
}

impl core::ops::BitOr for MapFlags {
    type Output = Self;
    fn bitor(self, _: Self) -> Self {
        Self
    }
}

struct FakeFrameAllocator;
impl FakeFrameAllocator {
    fn allocate_frame(&self) -> Option<PhysAddr> {
        // TODO: Usar phys::alloc
        None
    }
    fn deallocate_frame(&self, _: PhysAddr) {
        // TODO: Usar phys::free
    }
}

fn map_page_with_pmm<T>(_: u64, _: u64, _: MapFlags, _: &mut T) -> Result<(), u64> {
    // TODO: Implementar usando RMM mapper
    Ok(())
}

static FRAME_ALLOCATOR: Spinlock<FakeFrameAllocator> = Spinlock::new(FakeFrameAllocator);

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

/// Tamanho de uma página.
const PAGE_SIZE: usize = 4096;

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

impl DmaBuffer {
    /// Retorna o endereço virtual como ponteiro.
    pub fn as_ptr(&self) -> *const u8 {
        self.virt as *const u8
    }

    /// Retorna o endereço virtual como ponteiro mutável.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.virt as *mut u8
    }

    /// Retorna slice do buffer.
    ///
    /// # Safety
    /// Caller deve garantir acesso exclusivo durante DMA.
    pub unsafe fn as_slice(&self) -> &[u8] {
        core::slice::from_raw_parts(self.as_ptr(), self.size)
    }

    /// Retorna slice mutável do buffer.
    ///
    /// # Safety
    /// Caller deve garantir acesso exclusivo durante DMA.
    pub unsafe fn as_mut_slice(&self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size)
    }
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

    pool.initialized = true;
    pool.next_virt = DMA_POOL_START;

    crate::kinfo!("(DMA) Pool pronto:");
    crate::kinfo!("  Início:", DMA_POOL_START);
    crate::kinfo!("  Fim:", DMA_POOL_END);
}

/// Aloca um buffer DMA.
///
/// ## Parâmetros:
/// - `size`: Tamanho desejado (será arredondado para múltiplo de página)
/// - `owner`: ID do dispositivo alocador
/// - `direction`: Direção do fluxo de dados
///
/// ## Retorno:
/// Some(DmaBuffer) com ponteiros válidos, ou None em caso de falha.
///
/// ## Implementação
/// 1. Aloca frames físicos contíguos do PMM
/// 2. Mapeia na região virtual DMA_POOL_START+offset
/// 3. Retorna buffer com endereços virtual e físico
pub fn alloc(size: usize, owner: DeviceId, direction: DmaDirection) -> Option<DmaBuffer> {
    // Validações
    if size == 0 || size > MAX_DMA_SIZE {
        crate::kerror!("(DMA) Tamanho inválido:", size);
        return None;
    }

    // Arredonda para múltiplo de página
    let aligned_size = align_up(size, DMA_ALIGNMENT);
    let num_pages = aligned_size / PAGE_SIZE;

    let mut pool = DMA_POOL.lock();

    // Verifica espaço disponível na região virtual
    if pool.next_virt + aligned_size as u64 > DMA_POOL_END {
        crate::kerror!("(DMA) Pool virtual esgotado!");
        return None;
    }

    // Aloca frames físicos contíguos
    // TODO: Quando o MM for refatorado, considerar usar um alocador de
    // páginas contíguas dedicado para DMA (evita fragmentação)
    let mut pmm = FRAME_ALLOCATOR.lock();
    let mut frames: Vec<u64> = Vec::with_capacity(num_pages);

    for _ in 0..num_pages {
        if let Some(frame_addr) = pmm.allocate_frame() {
            frames.push(frame_addr.as_u64());
        } else {
            // Falha: libera frames já alocados
            crate::kerror!("(DMA) OOM ao alocar frames para DMA");
            for &frame in &frames {
                pmm.deallocate_frame(PhysAddr::new(frame));
            }
            return None;
        }
    }

    // Verifica se frames são contíguos (necessário para DMA)
    // TODO: Implementar alocação contígua garantida no PMM
    // Por enquanto, assumimos que vão ser contíguos se alocados em sequência
    let first_phys = frames[0];
    let mut is_contiguous = true;
    for (i, &frame) in frames.iter().enumerate().skip(1) {
        if frame != first_phys + (i as u64 * PAGE_SIZE as u64) {
            is_contiguous = false;
            break;
        }
    }

    if !is_contiguous {
        crate::kwarn!("(DMA) Frames não contíguos! DMA pode não funcionar corretamente");
        // TODO: Implementar fallback ou retry com alocação contígua
    }

    let virt_start = pool.next_virt;

    // Mapeia cada frame na região DMA
    for (i, &frame) in frames.iter().enumerate() {
        let page_virt = virt_start + (i as u64 * PAGE_SIZE as u64);

        // Mapeia com flags: Writable, No-Execute, No-Cache
        let flags = MapFlags::WRITABLE | MapFlags::NO_EXECUTE | MapFlags::NO_CACHE;

        if let Err(e) = map_page_with_pmm(page_virt, frame, flags, &mut pmm) {
            crate::kerror!("(DMA) Falha ao mapear página DMA:", e);
            // Libera frames alocados
            for &f in &frames {
                pmm.deallocate_frame(PhysAddr::new(f));
            }
            return None;
        }
    }

    // Zera o buffer (importante para segurança)
    unsafe {
        core::ptr::write_bytes(virt_start as *mut u8, 0, aligned_size);
    }

    let buffer = DmaBuffer {
        virt: virt_start,
        phys: first_phys,
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
        virt_start,
        "phys=",
        first_phys,
        "size=",
        aligned_size
    );

    Some(buffer)
}

/// Libera um buffer DMA.
///
/// Marca o buffer como livre e libera os frames físicos.
pub fn free(buffer: &DmaBuffer) {
    let mut pool = DMA_POOL.lock();
    let pmm = FRAME_ALLOCATOR.lock();

    for buf in pool.buffers.iter_mut() {
        if buf.virt == buffer.virt && buf.owner == buffer.owner {
            // Libera os frames físicos
            let num_pages = buf.size / PAGE_SIZE;
            for i in 0..num_pages {
                let frame_phys = buf.phys + (i as u64 * PAGE_SIZE as u64);
                pmm.deallocate_frame(PhysAddr::new(frame_phys));
            }

            // TODO: Desmapear as páginas virtuais (precisa de unmap_page)
            // Por enquanto, apenas marcamos como livre

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
    let pmm = FRAME_ALLOCATOR.lock();
    let mut freed_count = 0;
    let mut freed_size = 0;

    for buf in pool.buffers.iter_mut() {
        if buf.owner == owner && buf.in_use {
            // Libera os frames físicos
            let num_pages = buf.size / PAGE_SIZE;
            for i in 0..num_pages {
                let frame_phys = buf.phys + (i as u64 * PAGE_SIZE as u64);
                pmm.deallocate_frame(PhysAddr::new(frame_phys));
            }

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
#[inline]
pub fn phys_addr(buffer: &DmaBuffer) -> u64 {
    buffer.phys
}

/// Retorna o endereço virtual de um buffer DMA.
///
/// Usado pela CPU para acessar os dados.
#[inline]
pub fn virt_addr(buffer: &DmaBuffer) -> u64 {
    buffer.virt
}

/// Retorna ponteiro mutável para os dados do buffer.
///
/// ## Safety:
/// Caller deve garantir acesso exclusivo durante DMA.
#[inline]
pub unsafe fn as_mut_ptr(buffer: &DmaBuffer) -> *mut u8 {
    buffer.virt as *mut u8
}

/// Converte endereço virtual do DMA Pool para endereço físico.
///
/// ## Importante
/// Esta função só funciona para endereços dentro do DMA Pool!
/// Para outros endereços, use translate_addr() do VMM.
///
/// TODO: Quando o MM for refatorado, integrar com função unificada de conversão.
#[inline]
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    if !is_in_dma_pool(virt) {
        return None;
    }

    // Procura o buffer correspondente
    let pool = DMA_POOL.lock();
    for buf in pool.buffers.iter() {
        if buf.in_use && virt >= buf.virt && virt < buf.virt + buf.size as u64 {
            let offset = virt - buf.virt;
            return Some(buf.phys + offset);
        }
    }

    None
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
#[inline]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Verifica se um endereço está no DMA Pool.
#[inline]
pub fn is_in_dma_pool(addr: u64) -> bool {
    addr >= DMA_POOL_START && addr < DMA_POOL_END
}
