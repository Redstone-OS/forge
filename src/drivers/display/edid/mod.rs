//! # EDID Parser (Extended Display Identification Data)
//!
//! Interpreta informações enviadas pelo monitor (resoluções, fabricante, etc).
//!
//! ## Estrutura EDID:
//! - Bytes 0-7: Header
//! - Bytes 8-17: Vendor/Product ID
//! - Bytes 18-24: EDID Version/Basic Display
//! - Bytes 25-34: Color Characteristics
//! - Bytes 35-37: Established Timings
//! - Bytes 38-53: Standard Timings
//! - Bytes 54-125: Detailed Timing Descriptors
//! - Byte 126: Extension count
//! - Byte 127: Checksum

use alloc::string::String;
use alloc::vec::Vec;

/// Informações extraídas do EDID.
#[derive(Debug, Clone)]
pub struct EdidInfo {
    /// Código do fabricante (3 letras).
    pub manufacturer: [char; 3],
    /// Código do produto.
    pub product_code: u16,
    /// Número de série.
    pub serial_number: u32,
    /// Semana de fabricação.
    pub week: u8,
    /// Ano de fabricação.
    pub year: u16,
    /// Largura física em cm.
    pub width_cm: u8,
    /// Altura física em cm.
    pub height_cm: u8,
    /// Resolução preferida (largura).
    pub preferred_width: u16,
    /// Resolução preferida (altura).
    pub preferred_height: u16,
    /// Taxa de refresh preferida.
    pub preferred_refresh: u8,
    /// Nome do monitor (se disponível).
    pub monitor_name: Option<String>,
}

impl Default for EdidInfo {
    fn default() -> Self {
        Self {
            manufacturer: ['?', '?', '?'],
            product_code: 0,
            serial_number: 0,
            week: 0,
            year: 0,
            width_cm: 0,
            height_cm: 0,
            preferred_width: 0,
            preferred_height: 0,
            preferred_refresh: 60,
            monitor_name: None,
        }
    }
}

/// Parser de EDID.
pub struct EdidParser;

impl EdidParser {
    /// Header esperado do EDID.
    const HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

    /// Valida e interpreta um bloco EDID de 128 bytes.
    pub fn parse(data: &[u8]) -> Option<EdidInfo> {
        if data.len() < 128 {
            return None;
        }

        // Validar header
        if data[0..8] != Self::HEADER {
            crate::kwarn!("(EDID) Header inválido");
            return None;
        }

        // Validar checksum
        let checksum: u8 = data[0..128].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        if checksum != 0 {
            crate::kwarn!("(EDID) Checksum incorreto");
            return None;
        }

        let mut info = EdidInfo::default();

        // Extrair fabricante (bytes 8-9)
        let mfg = u16::from_be_bytes([data[8], data[9]]);
        info.manufacturer = [
            ((mfg >> 10) & 0x1F) as u8 + 0x40,
            ((mfg >> 5) & 0x1F) as u8 + 0x40,
            (mfg & 0x1F) as u8 + 0x40,
        ]
        .map(|c| c as char);

        // Produto (bytes 10-11)
        info.product_code = u16::from_le_bytes([data[10], data[11]]);

        // Serial (bytes 12-15)
        info.serial_number = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        // Data de fabricação (bytes 16-17)
        info.week = data[16];
        info.year = 1990 + data[17] as u16;

        // Tamanho físico (bytes 21-22)
        info.width_cm = data[21];
        info.height_cm = data[22];

        // Resolução preferida do primeiro detailed timing (bytes 54-71)
        if data[54] != 0 || data[55] != 0 {
            let h_active = data[56] as u16 | ((data[58] >> 4) as u16) << 8;
            let v_active = data[59] as u16 | ((data[61] >> 4) as u16) << 8;
            info.preferred_width = h_active;
            info.preferred_height = v_active;
        }

        // Buscar nome do monitor nos descriptors
        for i in 0..4 {
            let offset = 54 + i * 18;
            if data[offset] == 0 && data[offset + 1] == 0 && data[offset + 3] == 0xFC {
                // Monitor name descriptor
                let name_bytes = &data[offset + 5..offset + 18];
                let name: String = name_bytes
                    .iter()
                    .take_while(|&&c| c != 0x0A && c != 0x00)
                    .map(|&c| c as char)
                    .collect();
                if !name.is_empty() {
                    info.monitor_name = Some(name);
                }
            }
        }

        crate::kinfo!(
            "(EDID) Monitor: {} ({}x{})",
            info.monitor_name.as_deref().unwrap_or("Unknown"),
            info.preferred_width,
            info.preferred_height
        );

        Some(info)
    }

    /// Extrai todos os modos suportados do EDID.
    pub fn get_supported_modes(_data: &[u8]) -> Vec<(u16, u16, u8)> {
        // TODO: Parsear established timings, standard timings, detailed timings
        alloc::vec![
            (640, 480, 60),
            (800, 600, 60),
            (1024, 768, 60),
            (1280, 720, 60),
            (1920, 1080, 60),
        ]
    }
}
