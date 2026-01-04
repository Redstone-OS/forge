//! # xHCI Structures
//!
//! Estruturas de dados xHCI: TRBs, contextos, etc.

use super::types::*;

// =============================================================================
// TRB - TRANSFER REQUEST BLOCK
// =============================================================================

/// TRB genérico (16 bytes).
///
/// O TRB é a unidade fundamental de comunicação xHCI.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Trb {
    /// Parâmetro (significado depende do tipo).
    pub parameter: u64,
    /// Status (significado depende do tipo).
    pub status: u32,
    /// Control (inclui tipo e flags).
    pub control: u32,
}

impl Trb {
    /// Cria um TRB vazio.
    pub const fn new() -> Self {
        Self {
            parameter: 0,
            status: 0,
            control: 0,
        }
    }

    /// Retorna o tipo do TRB.
    pub fn trb_type(&self) -> u8 {
        ((self.control >> 10) & 0x3F) as u8
    }

    /// Define o tipo do TRB.
    pub fn set_type(&mut self, trb_type: u8) {
        self.control = (self.control & !0xFC00) | ((trb_type as u32) << 10);
    }

    /// Retorna o cycle bit.
    pub fn cycle(&self) -> bool {
        (self.control & 0x01) != 0
    }

    /// Define o cycle bit.
    pub fn set_cycle(&mut self, cycle: bool) {
        if cycle {
            self.control |= 0x01;
        } else {
            self.control &= !0x01;
        }
    }

    /// Cria um No-Op TRB.
    pub fn noop(cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.set_type(TRB_TYPE_NOOP);
        trb.set_cycle(cycle);
        trb
    }

    /// Cria um Link TRB.
    pub fn link(next_ring_addr: u64, toggle_cycle: bool, cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = next_ring_addr;
        trb.set_type(TRB_TYPE_LINK);
        trb.set_cycle(cycle);
        if toggle_cycle {
            trb.control |= 0x02; // Toggle Cycle bit
        }
        trb
    }

    /// Cria Enable Slot Command.
    pub fn enable_slot(cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.set_type(TRB_TYPE_ENABLE_SLOT);
        trb.set_cycle(cycle);
        trb
    }

    /// Cria Address Device Command.
    pub fn address_device(input_context_ptr: u64, slot_id: u8, bsr: bool, cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = input_context_ptr;
        trb.set_type(TRB_TYPE_ADDRESS_DEVICE);
        trb.control |= (slot_id as u32) << 24;
        if bsr {
            trb.control |= 1 << 9; // Block Set Address Request
        }
        trb.set_cycle(cycle);
        trb
    }
}

// =============================================================================
// CONTEXTOS
// =============================================================================

/// Slot Context (32 ou 64 bytes dependendo do controller).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SlotContext {
    /// Route String, Speed, etc.
    pub info1: u32,
    /// Max Exit Latency, Root Hub Port, etc.
    pub info2: u32,
    /// TT Info
    pub tt_info: u32,
    /// Slot State, Device Address
    pub state: u32,
    /// Reserved
    pub reserved: [u32; 4],
}

impl SlotContext {
    /// Retorna o estado do slot.
    pub fn slot_state(&self) -> u8 {
        ((self.state >> 27) & 0x1F) as u8
    }

    /// Retorna o device address.
    pub fn device_address(&self) -> u8 {
        (self.state & 0xFF) as u8
    }

    /// Define a velocidade do dispositivo.
    pub fn set_speed(&mut self, speed: u8) {
        self.info1 = (self.info1 & !0x00F00000) | ((speed as u32) << 20);
    }

    /// Define a porta root hub.
    pub fn set_root_hub_port(&mut self, port: u8) {
        self.info2 = (self.info2 & !0x00FF0000) | ((port as u32) << 16);
    }

    /// Define context entries.
    pub fn set_context_entries(&mut self, entries: u8) {
        self.info1 = (self.info1 & !0xF8000000) | ((entries as u32) << 27);
    }
}

/// Endpoint Context (32 ou 64 bytes).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct EndpointContext {
    /// Endpoint State, Mult, MaxPStreams, etc.
    pub info1: u32,
    /// Max Packet Size, Max Burst Size, etc.
    pub info2: u32,
    /// TR Dequeue Pointer
    pub tr_dequeue_ptr: u64,
    /// Average TRB Length, Max ESIT Payload
    pub info3: u32,
    /// Reserved
    pub reserved: [u32; 3],
}

impl EndpointContext {
    /// Retorna o estado do endpoint.
    pub fn ep_state(&self) -> u8 {
        (self.info1 & 0x07) as u8
    }

    /// Define o tipo do endpoint.
    pub fn set_ep_type(&mut self, ep_type: u8) {
        self.info1 = (self.info1 & !0x38) | ((ep_type as u32) << 3);
    }

    /// Define max packet size.
    pub fn set_max_packet_size(&mut self, size: u16) {
        self.info2 = (self.info2 & !0xFFFF0000) | ((size as u32) << 16);
    }

    /// Define o TR Dequeue Pointer.
    pub fn set_tr_dequeue_ptr(&mut self, ptr: u64, dcs: bool) {
        self.tr_dequeue_ptr = (ptr & !0x0F) | if dcs { 1 } else { 0 };
    }

    /// Define average TRB length.
    pub fn set_average_trb_length(&mut self, len: u16) {
        self.info3 = (self.info3 & !0xFFFF) | (len as u32);
    }

    /// Define intervalo (para interrupt endpoints).
    pub fn set_interval(&mut self, interval: u8) {
        self.info1 = (self.info1 & !0xFF0000) | ((interval as u32) << 16);
    }
}

/// Input Control Context.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct InputControlContext {
    /// Drop Context Flags
    pub drop_flags: u32,
    /// Add Context Flags
    pub add_flags: u32,
    /// Reserved
    pub reserved: [u32; 5],
    /// Configuration Value, Interface Number, Alternate Setting
    pub config: u32,
}

impl InputControlContext {
    /// Adiciona um contexto.
    pub fn add_context(&mut self, index: u8) {
        self.add_flags |= 1 << index;
    }

    /// Remove um contexto.
    pub fn drop_context(&mut self, index: u8) {
        self.drop_flags |= 1 << index;
    }
}

// =============================================================================
// EVENT RING SEGMENT TABLE ENTRY
// =============================================================================

/// Entrada na Event Ring Segment Table.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ErstEntry {
    /// Endereço base do segmento.
    pub ring_segment_base: u64,
    /// Tamanho do segmento (em TRBs).
    pub ring_segment_size: u16,
    /// Reservado.
    pub reserved: u16,
    /// Reservado.
    pub reserved2: u32,
}
