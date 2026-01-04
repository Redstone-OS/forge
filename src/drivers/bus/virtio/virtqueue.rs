//! # VirtQueue - Fila de Requisições VirtIO
//!
//! Implementa as **VirtQueues** - o mecanismo central de comunicação
//! entre driver e dispositivo VirtIO.
//!
//! ## Estrutura de uma VirtQueue:
//! ```text
//! +------------------+
//! | Descriptor Table | <- Lista de buffers
//! +------------------+
//! | Available Ring   | <- Índices disponíveis para dispositivo
//! +------------------+
//! | Used Ring        | <- Índices processados pelo dispositivo
//! +------------------+
//! ```
//!
//! ## Fluxo de Uso:
//! 1. Driver aloca buffer e adiciona descritor
//! 2. Driver adiciona índice ao Available Ring
//! 3. Driver notifica dispositivo
//! 4. Dispositivo processa e adiciona ao Used Ring
//! 5. Dispositivo gera interrupção
//! 6. Driver lê do Used Ring
//!
//! ## STUB:
//! Implementação parcial. Aloca estruturas mas não funciona completamente.

// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::types::*;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::base::dma::{self, DmaBuffer, DmaDirection};
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Tamanho padrão de uma VirtQueue.
pub const VIRTQUEUE_DEFAULT_SIZE: u16 = 256;

/// Alinhamento requerido para estruturas de VirtQueue.
pub const VIRTQUEUE_ALIGN: usize = 4096;

// =============================================================================
// ESTRUTURAS DE DADOS DA VIRTQUEUE
// =============================================================================

/// Descritor de buffer na VirtQueue.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqDesc {
    /// Endereço físico do buffer.
    pub addr: u64,

    /// Tamanho do buffer em bytes.
    pub len: u32,

    /// Flags do descritor.
    pub flags: u16,

    /// Próximo descritor na cadeia (se VIRTQ_DESC_F_NEXT).
    pub next: u16,
}

/// Estrutura do Available Ring (driver → device).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqAvail {
    /// Flags do ring.
    pub flags: u16,

    /// Índice do próximo slot a ser escrito pelo driver.
    pub idx: u16,
    // Seguido por: ring[queue_size] e used_event
}

/// Estrutura do Used Ring (device → driver).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqUsed {
    /// Flags do ring.
    pub flags: u16,

    /// Índice do próximo slot a ser lido pelo driver.
    pub idx: u16,
    // Seguido por: ring[queue_size] e avail_event
}

/// Elemento do Used Ring.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqUsedElem {
    /// Índice do descritor processado.
    pub id: u32,

    /// Total de bytes escritos pelo dispositivo.
    pub len: u32,
}

// =============================================================================
// VIRTQUEUE
// =============================================================================

/// Representa uma VirtQueue completa.
pub struct VirtQueue {
    /// Número da queue (índice).
    pub index: u16,

    /// Tamanho da queue (número de descritores).
    pub size: u16,

    /// Buffer DMA contendo as estruturas.
    pub dma_buffer: Option<DmaBuffer>,

    /// Ponteiro para tabela de descritores.
    desc_table: u64,

    /// Ponteiro para Available Ring.
    avail_ring: u64,

    /// Ponteiro para Used Ring.
    used_ring: u64,

    /// Próximo índice livre na tabela de descritores.
    free_head: u16,

    /// Último índice processado do Used Ring.
    last_used_idx: u16,

    /// Número de descritores em uso.
    num_used: u16,

    /// Lista de descritores livres.
    free_list: Vec<u16>,
}

impl VirtQueue {
    /// Cria uma nova VirtQueue.
    ///
    /// ## Parâmetros:
    /// - `index`: Número da queue
    /// - `size`: Número de descritores (potência de 2)
    ///
    /// ## STUB:
    /// Aloca estruturas mas não as inicializa completamente.
    pub fn new(index: u16, size: u16) -> Option<Self> {
        crate::kwarn!("(VirtQueue) new() parcialmente implementado");

        // Calcula tamanho necessário para o buffer DMA
        let desc_size = size as usize * core::mem::size_of::<VirtqDesc>();
        let avail_size = 6 + size as usize * 2; // flags + idx + ring[] + used_event
        let used_size = 6 + size as usize * core::mem::size_of::<VirtqUsedElem>(); // flags + idx + ring[] + avail_event

        // TODO: Revisar no futuro
        #[allow(unused_variables)]
        let total_size = desc_size + avail_size + used_size;

        // TODO: Precisa alinhar cada seção corretamente
        // Por enquanto, apenas calcula

        // Aloca buffer DMA
        // let dma = dma::alloc(total_size, device_id, DmaDirection::Bidirectional)?;

        // Cria lista de descritores livres
        // TODO: Revisar no futuro
        #[allow(unused_mut)]
        let mut free_list: Vec<u16> = (0..size).collect();

        Some(Self {
            index,
            size,
            dma_buffer: None, // TODO: Alocar
            desc_table: 0,
            avail_ring: 0,
            used_ring: 0,
            free_head: 0,
            last_used_idx: 0,
            num_used: 0,
            free_list,
        })
    }

    /// Aloca um descritor livre.
    pub fn alloc_desc(&mut self) -> Option<u16> {
        self.free_list.pop()
    }

    /// Libera um descritor.
    pub fn free_desc(&mut self, desc_idx: u16) {
        self.free_list.push(desc_idx);
    }

    /// Retorna número de descritores disponíveis.
    pub fn num_free(&self) -> u16 {
        self.free_list.len() as u16
    }

    /// Adiciona um buffer à queue.
    ///
    /// ## STUB:
    /// Não adiciona realmente, apenas loga.
    pub fn add_buffer(
        &mut self,
        out_bufs: &[(u64, u32)], // (addr, len) - buffers para dispositivo ler
        in_bufs: &[(u64, u32)],  // (addr, len) - buffers para dispositivo escrever
    ) -> Option<u16> {
        crate::kwarn!("(VirtQueue) add_buffer() stub");

        // Verifica se há descritores suficientes
        let total_descs = out_bufs.len() + in_bufs.len();
        if total_descs > self.num_free() as usize {
            crate::kerror!("(VirtQueue) Sem descritores disponíveis");
            return None;
        }

        // TODO: Implementar
        // 1. Alocar descritores
        // 2. Preencher com addr, len, flags
        // 3. Encadear se necessário
        // 4. Adicionar cabeça ao avail ring
        // 5. Incrementar avail.idx

        Some(0) // Retorna índice do primeiro descritor
    }

    /// Processa buffers completados pelo dispositivo.
    ///
    /// ## STUB:
    /// Não processa realmente.
    pub fn poll_used(&mut self) -> Option<(u16, u32)> {
        crate::kwarn!("(VirtQueue) poll_used() stub");

        // TODO: Implementar
        // 1. Ler used.idx
        // 2. Comparar com last_used_idx
        // 3. Se diferente, há buffers processados
        // 4. Ler used.ring[last_used_idx % size]
        // 5. Incrementar last_used_idx
        // 6. Retornar (desc_idx, bytes_written)

        None
    }

    /// Retorna endereços físicos das estruturas para configurar no dispositivo.
    pub fn get_addresses(&self) -> (u64, u64, u64) {
        (self.desc_table, self.avail_ring, self.used_ring)
    }
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Calcula tamanho total necessário para uma VirtQueue.
pub fn virtqueue_size(queue_size: u16) -> usize {
    let desc_size = queue_size as usize * core::mem::size_of::<VirtqDesc>();
    let avail_size = 6 + queue_size as usize * 2;
    let used_size = 6 + queue_size as usize * core::mem::size_of::<VirtqUsedElem>();

    // Alinhamentos
    let desc_end = desc_size;
    let avail_end = align_up(desc_end + avail_size, 4);
    let used_end = align_up(avail_end + used_size, VIRTQUEUE_ALIGN);

    used_end
}

/// Alinha valor para cima.
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
