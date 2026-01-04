//! # Network Device Traits
//!
//! Define as interfaces que todos os drivers de rede devem implementar.
//!
//! ## Trait Principal: `NetworkDevice`
//! Interface base para qualquer dispositivo de rede.
//!
//! ## Traits Especializadas:
//! - `EthernetDevice`: Para NICs Ethernet
//! - `WifiDevice`: Para adaptadores WiFi

use super::NetworkStats;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// TIPOS COMUNS
// =============================================================================

/// Endereço MAC (6 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Endereço MAC de broadcast.
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

    /// Cria a partir de 6 bytes.
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Verifica se é broadcast.
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    /// Verifica se é multicast.
    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 0x01) != 0
    }

    /// Verifica se é unicast.
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    /// Formata como string "XX:XX:XX:XX:XX:XX".
    pub fn to_string(&self) -> String {
        alloc::format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5]
        )
    }
}

/// Estado do link físico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// Link inativo (cabo desconectado).
    Down,
    /// Link ativo.
    Up,
    /// Negociando (autonegotiation em andamento).
    Negotiating,
    /// Desconhecido.
    Unknown,
}

/// Velocidade do link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkSpeed {
    /// 10 Mbps.
    Speed10,
    /// 100 Mbps.
    Speed100,
    /// 1 Gbps.
    Speed1000,
    /// 2.5 Gbps.
    Speed2500,
    /// 5 Gbps.
    Speed5000,
    /// 10 Gbps.
    Speed10000,
    /// Desconhecida.
    Unknown,
}

impl LinkSpeed {
    /// Retorna velocidade em Mbps.
    pub fn mbps(&self) -> u32 {
        match self {
            Self::Speed10 => 10,
            Self::Speed100 => 100,
            Self::Speed1000 => 1000,
            Self::Speed2500 => 2500,
            Self::Speed5000 => 5000,
            Self::Speed10000 => 10000,
            Self::Unknown => 0,
        }
    }
}

/// Modo duplex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplexMode {
    Half,
    Full,
    Unknown,
}

// =============================================================================
// TRAIT PRINCIPAL: NETWORK DEVICE
// =============================================================================

/// Interface base para dispositivos de rede.
///
/// Todo driver de rede deve implementar esta trait.
pub trait NetworkDevice: Send + Sync {
    /// Retorna nome do dispositivo (ex: "eth0", "wlan0").
    fn name(&self) -> &str;

    /// Retorna endereço MAC.
    fn mac_address(&self) -> MacAddress;

    /// Retorna MTU (Maximum Transmission Unit).
    fn mtu(&self) -> u16;

    /// Define MTU.
    fn set_mtu(&self, mtu: u16) -> bool;

    /// Verifica se o link está ativo.
    fn is_link_up(&self) -> bool;

    /// Retorna estado detalhado do link.
    fn link_state(&self) -> LinkState;

    /// Retorna velocidade do link.
    fn link_speed(&self) -> LinkSpeed;

    /// Retorna estatísticas.
    fn get_stats(&self) -> NetworkStats;

    /// Reseta estatísticas.
    fn reset_stats(&self);

    /// Transmite um pacote.
    ///
    /// ## Parâmetros:
    /// - `data`: Buffer contendo o frame Ethernet completo
    ///
    /// ## Retorno:
    /// - Ok(bytes): Número de bytes transmitidos
    /// - Err: Erro de transmissão
    fn transmit(&self, data: &[u8]) -> Result<usize, NetworkError>;

    /// Recebe um pacote (polling).
    ///
    /// ## Parâmetros:
    /// - `buffer`: Buffer para receber o frame
    ///
    /// ## Retorno:
    /// - Ok(Some(bytes)): Pacote recebido
    /// - Ok(None): Nenhum pacote disponível
    /// - Err: Erro de recepção
    fn receive(&self, buffer: &mut [u8]) -> Result<Option<usize>, NetworkError>;

    /// Habilita o dispositivo.
    fn enable(&self) -> bool;

    /// Desabilita o dispositivo.
    fn disable(&self);

    /// Verifica se está habilitado.
    fn is_enabled(&self) -> bool;

    /// Habilita modo promíscuo.
    fn set_promiscuous(&self, enabled: bool);

    /// Adiciona endereço multicast ao filtro.
    fn add_multicast(&self, mac: MacAddress);

    /// Remove endereço multicast do filtro.
    fn remove_multicast(&self, mac: MacAddress);
}

// =============================================================================
// ERROS DE REDE
// =============================================================================

/// Erros de operações de rede.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    /// Dispositivo não está habilitado.
    NotEnabled,
    /// Link está inativo.
    LinkDown,
    /// Buffer muito pequeno.
    BufferTooSmall,
    /// Pacote muito grande.
    PacketTooLarge,
    /// Fila de TX cheia.
    TxQueueFull,
    /// Fila de RX vazia.
    RxQueueEmpty,
    /// Timeout.
    Timeout,
    /// Erro de hardware.
    HardwareError,
    /// Operação não suportada.
    NotSupported,
    /// Erro genérico.
    Unknown,
}

impl NetworkError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotEnabled => "Not Enabled",
            Self::LinkDown => "Link Down",
            Self::BufferTooSmall => "Buffer Too Small",
            Self::PacketTooLarge => "Packet Too Large",
            Self::TxQueueFull => "TX Queue Full",
            Self::RxQueueEmpty => "RX Queue Empty",
            Self::Timeout => "Timeout",
            Self::HardwareError => "Hardware Error",
            Self::NotSupported => "Not Supported",
            Self::Unknown => "Unknown",
        }
    }
}

// =============================================================================
// TRAITS ESPECIALIZADAS
// =============================================================================

/// Interface para dispositivos Ethernet.
pub trait EthernetDevice: NetworkDevice {
    /// Retorna modo duplex.
    fn duplex(&self) -> DuplexMode;

    /// Habilita/desabilita autonegotiation.
    fn set_autoneg(&self, enabled: bool);

    /// Força velocidade e duplex específicos.
    fn set_speed_duplex(&self, speed: LinkSpeed, duplex: DuplexMode);
}

/// Interface para dispositivos WiFi.
pub trait WifiDevice: NetworkDevice {
    /// Escaneia redes disponíveis.
    fn scan_networks(&self) -> Vec<WifiNetwork>;

    /// Conecta a uma rede.
    fn connect(&self, ssid: &str, password: &str) -> bool;

    /// Desconecta da rede atual.
    fn disconnect(&self);

    /// Retorna SSID da rede conectada.
    fn current_ssid(&self) -> Option<String>;

    /// Retorna força do sinal (dBm).
    fn signal_strength(&self) -> i8;
}

/// Informações de uma rede WiFi.
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: MacAddress,
    pub channel: u8,
    pub signal_strength: i8,
    pub security: WifiSecurity,
}

/// Tipos de segurança WiFi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    Wep,
    WpaPsk,
    Wpa2Psk,
    Wpa3Psk,
    WpaEnterprise,
    Wpa2Enterprise,
}
