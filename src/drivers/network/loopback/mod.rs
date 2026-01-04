//! # Loopback Network Device
//!
//! Implementa o dispositivo de **loopback** - a interface de rede local
//! que redireciona todo tráfego de volta para o próprio sistema.
//!
//! ## Características:
//! - Endereço: 127.0.0.1 (IPv4), ::1 (IPv6)
//! - Nome: "lo"
//! - MTU: 65536 (máximo)
//! - Sempre ativo
//!
//! ## Uso:
//! - Comunicação entre processos locais via sockets
//! - Testes de rede sem hardware
//! - Serviços que escutam em localhost
//!
//! ## STUB:
//! Estrutura básica definida. Integração com stack TCP/IP pendente.

use super::traits::*;
use super::NetworkStats;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Nome do dispositivo loopback.
pub const LOOPBACK_NAME: &str = "lo";

/// MTU do loopback (máximo possível).
pub const LOOPBACK_MTU: u16 = 65536;

/// MAC address fictício para loopback.
pub const LOOPBACK_MAC: MacAddress = MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

// =============================================================================
// ESTRUTURA DO DISPOSITIVO
// =============================================================================

/// Dispositivo de loopback.
pub struct LoopbackDevice {
    /// Estatísticas.
    stats: Spinlock<NetworkStats>,

    /// Fila de pacotes (loopback).
    queue: Spinlock<Vec<Vec<u8>>>,

    /// Dispositivo está habilitado?
    enabled: Spinlock<bool>,

    /// MTU atual.
    mtu: Spinlock<u16>,
}

impl LoopbackDevice {
    /// Cria o dispositivo loopback.
    pub fn new() -> Self {
        Self {
            stats: Spinlock::new(NetworkStats::default()),
            queue: Spinlock::new(Vec::new()),
            enabled: Spinlock::new(true), // Sempre habilitado
            mtu: Spinlock::new(LOOPBACK_MTU),
        }
    }
}

impl NetworkDevice for LoopbackDevice {
    fn name(&self) -> &str {
        LOOPBACK_NAME
    }

    fn mac_address(&self) -> MacAddress {
        LOOPBACK_MAC
    }

    fn mtu(&self) -> u16 {
        *self.mtu.lock()
    }

    fn set_mtu(&self, mtu: u16) -> bool {
        *self.mtu.lock() = mtu;
        true
    }

    fn is_link_up(&self) -> bool {
        true // Loopback sempre ativo
    }

    fn link_state(&self) -> LinkState {
        LinkState::Up
    }

    fn link_speed(&self) -> LinkSpeed {
        LinkSpeed::Unknown // Não aplicável
    }

    fn get_stats(&self) -> NetworkStats {
        self.stats.lock().clone()
    }

    fn reset_stats(&self) {
        *self.stats.lock() = NetworkStats::default();
    }

    fn transmit(&self, data: &[u8]) -> Result<usize, NetworkError> {
        if !*self.enabled.lock() {
            return Err(NetworkError::NotEnabled);
        }

        let len = data.len();

        // Loopback: adiciona na fila de recepção
        self.queue.lock().push(data.to_vec());

        // Atualiza estatísticas
        let mut stats = self.stats.lock();
        stats.tx_packets += 1;
        stats.tx_bytes += len as u64;
        stats.rx_packets += 1; // Também recebe
        stats.rx_bytes += len as u64;

        Ok(len)
    }

    fn receive(&self, buffer: &mut [u8]) -> Result<Option<usize>, NetworkError> {
        if !*self.enabled.lock() {
            return Err(NetworkError::NotEnabled);
        }

        let mut queue = self.queue.lock();

        if let Some(packet) = queue.pop() {
            if buffer.len() < packet.len() {
                // Pacote não cabe - devolve à fila e retorna erro
                queue.push(packet);
                return Err(NetworkError::BufferTooSmall);
            }

            let len = packet.len();
            buffer[..len].copy_from_slice(&packet);
            Ok(Some(len))
        } else {
            Ok(None)
        }
    }

    fn enable(&self) -> bool {
        *self.enabled.lock() = true;
        true
    }

    fn disable(&self) {
        *self.enabled.lock() = false;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock()
    }

    fn set_promiscuous(&self, _enabled: bool) {
        // Não aplicável para loopback
    }

    fn add_multicast(&self, _mac: MacAddress) {
        // Não aplicável para loopback
    }

    fn remove_multicast(&self, _mac: MacAddress) {
        // Não aplicável para loopback
    }
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static LOOPBACK: Spinlock<Option<Arc<LoopbackDevice>>> = Spinlock::new(None);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o dispositivo loopback.
pub fn init() {
    crate::kinfo!("(Loopback) Inicializando dispositivo loopback...");

    let device = Arc::new(LoopbackDevice::new());
    *LOOPBACK.lock() = Some(device.clone());

    // Registra no subsistema de rede
    super::register_device(device);

    crate::kinfo!("(Loopback) Dispositivo 'lo' criado");
}

/// Desliga o loopback.
pub fn shutdown() {
    crate::kinfo!("(Loopback) Shutdown");
}

/// Retorna referência ao dispositivo loopback.
pub fn get_device() -> Option<Arc<LoopbackDevice>> {
    LOOPBACK.lock().clone()
}
