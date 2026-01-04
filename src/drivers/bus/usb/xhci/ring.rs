//! # xHCI Rings
//!
//! Implementação de Command, Event e Transfer Rings.

use super::structs::Trb;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::types::*;

// =============================================================================
// RING
// =============================================================================

/// Ring xHCI (Command, Event ou Transfer).
pub struct XhciRing {
    /// Endereço físico do ring.
    pub phys_addr: u64,

    /// Endereço virtual do ring.
    pub virt_addr: u64,

    /// Número de TRBs no ring.
    pub size: usize,

    /// Índice do próximo TRB a ser enfileirado (producer).
    pub enqueue_index: usize,

    /// Índice do próximo TRB a ser processado (consumer).
    pub dequeue_index: usize,

    /// Cycle bit atual.
    pub cycle: bool,

    /// Tipo de ring.
    pub ring_type: RingType,
}

/// Tipo de ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingType {
    Command,
    Event,
    Transfer,
}

impl XhciRing {
    /// Cria um novo ring.
    ///
    /// ## STUB:
    /// Não aloca memória real.
    pub fn new(ring_type: RingType, size: usize) -> Option<Self> {
        crate::kwarn!("(xHCI Ring) new() stub - sem alocação real");

        // TODO: Alocar memória física contigua via DMA pool

        Some(Self {
            phys_addr: 0,
            virt_addr: 0,
            size,
            enqueue_index: 0,
            dequeue_index: 0,
            cycle: true,
            ring_type,
        })
    }

    /// Enfileira um TRB.
    ///
    /// ## STUB:
    /// Não escreve realmente.
    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    pub fn enqueue(&mut self, trb: Trb) -> bool {
        if self.is_full() {
            return false;
        }

        crate::kwarn!("(xHCI Ring) enqueue() stub");

        // TODO: Escrever TRB na memória
        // TODO: Atualizar cycle bit
        // TODO: Se chegou ao fim, adicionar Link TRB

        self.enqueue_index = (self.enqueue_index + 1) % self.size;

        // Se voltou ao início, toggle cycle
        if self.enqueue_index == 0 {
            self.cycle = !self.cycle;
        }

        true
    }

    /// Desenfileira um TRB (para Event Ring).
    ///
    /// ## STUB:
    /// Não lê realmente.
    pub fn dequeue(&mut self) -> Option<Trb> {
        if self.is_empty() {
            return None;
        }

        crate::kwarn!("(xHCI Ring) dequeue() stub");

        // TODO: Ler TRB da memória
        // TODO: Verificar cycle bit

        self.dequeue_index = (self.dequeue_index + 1) % self.size;

        Some(Trb::new())
    }

    /// Verifica se o ring está cheio.
    pub fn is_full(&self) -> bool {
        let next = (self.enqueue_index + 1) % self.size;
        next == self.dequeue_index
    }

    /// Verifica se o ring está vazio.
    pub fn is_empty(&self) -> bool {
        self.enqueue_index == self.dequeue_index
    }

    /// Retorna número de TRBs disponíveis.
    pub fn available(&self) -> usize {
        if self.enqueue_index >= self.dequeue_index {
            self.size - (self.enqueue_index - self.dequeue_index) - 1
        } else {
            self.dequeue_index - self.enqueue_index - 1
        }
    }
}
