//! # 802.11 Wireless Frame Headers
//!
//! Contém definições de cabeçalhos e tipos para quadros Wi-Fi.

/// Tipos de quadros 802.11
pub enum FrameType {
    Management = 0x00,
    Control = 0x01,
    Data = 0x02,
    Reserved = 0x03,
}

/// Cabeçalho comum de um quadro 802.11
#[repr(C, packed)]
pub struct Dot11Header {
    pub frame_control: u16,
    pub duration_id: u16,
    pub addr1: [u8; 6],
    pub addr2: [u8; 6],
    pub addr3: [u8; 6],
    pub seq_ctrl: u16,
}

impl Dot11Header {
    /// STUB: Validação de CRC/FCS do quadro
    pub fn verify_fcs(&self) -> bool {
        // TODO: Implementar cálculo de CRC32 conforme padrão 802.11
        true
    }
}
