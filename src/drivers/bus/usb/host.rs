//! # USB Host Controller Interface
//!
//! Trait e tipos para host controllers USB.

// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::device::UsbDevice;
use super::types::*;

// =============================================================================
// TRAIT DE HOST CONTROLLER
// =============================================================================

/// Interface que todo host controller USB deve implementar.
///
/// ## STUB:
/// Métodos têm implementações padrão que emitem warnings.
pub trait UsbHostController: Send + Sync {
    /// Retorna nome do controller.
    fn name(&self) -> &'static str;

    /// Retorna número de portas root.
    fn port_count(&self) -> u8;

    /// Verifica status de uma porta.
    fn port_status(&self, port: u8) -> PortStatus;

    /// Realiza reset de uma porta.
    fn port_reset(&self, port: u8) -> bool;

    /// Habilita uma porta.
    fn port_enable(&self, port: u8);

    /// Desabilita uma porta.
    fn port_disable(&self, port: u8);

    /// Aloca um slot/address para novo dispositivo.
    fn allocate_address(&self) -> Option<u8>;

    /// Libera um endereço.
    fn free_address(&self, address: u8);

    /// Executa uma control transfer.
    fn control_transfer(
        &self,
        address: u8,
        setup: &UsbSetupPacket,
        data: Option<&mut [u8]>,
    ) -> Result<usize, UsbError>;

    /// Executa uma bulk transfer.
    fn bulk_transfer(
        &self,
        address: u8,
        endpoint: u8,
        data: &mut [u8],
        direction: UsbDirection,
    ) -> Result<usize, UsbError>;

    /// Executa uma interrupt transfer.
    fn interrupt_transfer(
        &self,
        address: u8,
        endpoint: u8,
        data: &mut [u8],
        direction: UsbDirection,
    ) -> Result<usize, UsbError>;

    /// Realiza reset do controller.
    fn reset(&self) -> bool;

    /// Desliga o controller.
    fn shutdown(&self);
}

// =============================================================================
// STATUS DE PORTA
// =============================================================================

/// Status de uma porta USB.
#[derive(Debug, Clone, Copy, Default)]
pub struct PortStatus {
    /// Dispositivo conectado.
    pub connected: bool,

    /// Porta habilitada.
    pub enabled: bool,

    /// Reset em andamento.
    pub resetting: bool,

    /// Velocidade do dispositivo (se conectado).
    pub speed: Option<UsbSpeed>,

    /// Houve mudança de conexão.
    pub connect_change: bool,

    /// Houve mudança de enable.
    pub enable_change: bool,

    /// Porta em overcurrent.
    pub overcurrent: bool,
}

// =============================================================================
// ERROS USB
// =============================================================================

/// Erros de operações USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    /// Dispositivo não respondeu (timeout).
    Timeout,

    /// Dispositivo enviou STALL.
    Stall,

    /// Erro de CRC ou bitstuff.
    DataError,

    /// Buffer muito pequeno.
    BufferTooSmall,

    /// Dispositivo não existe.
    NoDevice,

    /// Endpoint inválido.
    InvalidEndpoint,

    /// Transfer cancelada.
    Cancelled,

    /// Erro do host controller.
    HostError,

    /// Erro desconhecido.
    Unknown,
}

impl UsbError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timeout => "Timeout",
            Self::Stall => "Stall",
            Self::DataError => "Data Error",
            Self::BufferTooSmall => "Buffer Too Small",
            Self::NoDevice => "No Device",
            Self::InvalidEndpoint => "Invalid Endpoint",
            Self::Cancelled => "Cancelled",
            Self::HostError => "Host Error",
            Self::Unknown => "Unknown",
        }
    }
}
