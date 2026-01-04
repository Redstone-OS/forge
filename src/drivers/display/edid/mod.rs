//! # EDID Parser (Extended Display Identification Data)
//!
//! Responsável por interpretar as informações enviadas pelo monitor
//! (resoluções suportadas, fabricante, tamanho físico).

pub mod structs;

use self::structs::{DetailedTiming, EdidBlock};

pub struct EdidInfo {
    pub manufacturer: [char; 3],
    pub product_code: u16,
    pub width_cm: u8,
    pub height_cm: u8,
    pub preferred_width: u16,
    pub preferred_height: u16,
}

pub struct EdidParser;

impl EdidParser {
    /// Valida e interpreta um bloco EDID de 128 bytes
    pub fn parse(data: &[u8]) -> Option<EdidInfo> {
        if data.len() < 128 {
            return None;
        }

        let edid = unsafe { &*(data.as_ptr() as *const EdidBlock) };

        // 1. Validar Header (00 FF FF FF FF FF FF 00)
        let expected_header: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        if edid.header != expected_header {
            crate::kwarn!("(EDID) Header inválido detectado.");
            return None;
        }

        // 2. Validar Checksum
        let mut sum: u8 = 0;
        for &byte in data[0..128].iter() {
            sum = sum.wrapping_add(byte);
        }
        if sum != 0 {
            crate::kwarn!("(EDID) Checksum incorreto.");
            return None;
        }

        // 3. Extrair ID do Fabricante (3 letras codificadas em 5 bits)
        let mfg = edid.mfg_id.swap_bytes();
        let char1 = ((mfg >> 10) & 0x1F) as u8 + 0x40;
        let char2 = ((mfg >> 5) & 0x1F) as u8 + 0x40;
        let char3 = (mfg & 0x1F) as u8 + 0x40;

        // 4. Extrair Resolução Preferida (Primeiro descriptor detalhado)
        let preferred = &edid.detailed_timings[0];
        let width =
            preferred.h_active_low as u16 | (((preferred.h_active_blank_high >> 4) as u16) << 8);
        let height =
            preferred.v_active_low as u16 | (((preferred.v_active_blank_high >> 4) as u16) << 8);

        crate::kinfo!("(EDID) Monitor Detectado:", 0);
        crate::kdebug!("  Fabricante:", (char1 as char).to_string().as_str()); // Placeholder para log string
        crate::kinfo!("  Resolução Nativa:", width as u64);
        crate::kinfo!("  x", height as u64);

        Some(EdidInfo {
            manufacturer: [char1 as char, char2 as char, char3 as char],
            product_code: edid.prod_code,
            width_cm: edid.screen_size_h,
            height_cm: edid.screen_size_v,
            preferred_width: width,
            preferred_height: height,
        })
    }
}
