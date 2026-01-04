//! # Network Adapter Traits
//!
//! Abstrações comuns para placas de rede (NICs).

use alloc::vec::Vec;

/// Endereço MAC (48 bits)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    pub const ZERO: Self = Self([0, 0, 0, 0, 0, 0]);
}

/// Status do link de rede
pub enum LinkStatus {
    Up,
    Down,
    Unknown,
}

/// Interface unificada para drivers de rede
pub trait NetworkAdapter {
    /// Nome amigável do adaptador
    fn name(&self) -> &'static str;

    /// Retorna o endereço MAC da placa
    fn mac_address(&self) -> MacAddress;

    /// Status atual da conexão física
    fn link_status(&self) -> LinkStatus;

    /// Transmite um pacote bruto (Ethernet Frame)
    fn transmit(&self, packet: &[u8]) -> Result<(), &'static str>;

    /// Tenta receber um pacote da fila
    fn receive(&self) -> Option<Vec<u8>>;
}
