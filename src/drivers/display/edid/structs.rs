//! # EDID Data Structures
//!
//! Estruturas de dados para o padrão Extended Display Identification Data (EDID).
//! Baseado na especificação VESA EDID v1.3/v1.4.

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EdidBlock {
    pub header: [u8; 8],                       // 00-07: 00 FF FF FF FF FF FF 00
    pub mfg_id: u16,                           // 08-09: Manufacturer ID
    pub prod_code: u16,                        // 0A-0B: Product Code
    pub serial: u32,                           // 0C-0F: Serial Number
    pub mfg_week: u8,                          // 10: Week of Manufacture
    pub mfg_year: u8,                          // 11: Year of Manufacture (Year + 1990)
    pub edid_version: u8,                      // 12: EDID Version (usually 1)
    pub edid_revision: u8,                     // 13: EDID Revision (usually 3 or 4)
    pub video_input: u8,                       // 14: Digital/Analog input params
    pub screen_size_h: u8,                     // 15: Width in cm
    pub screen_size_v: u8,                     // 16: Height in cm
    pub gamma: u8,                             // 17: Gamma (Value * 100 + 100)
    pub features: u8,                          // 18: DPMS, Color type, etc.
    pub color_coords: [u8; 10],                // 19-22: Red/Green low bits, etc.
    pub established_timings: [u8; 3],          // 23-25: 720x400@70, 640x480@60, etc.
    pub standard_timings: [u8; 16],            // 26-35: 8 identified standard timings
    pub detailed_timings: [DetailedTiming; 4], // 36-7D: 4 descriptors
    pub extension_count: u8,                   // 7E: Number of extensions
    pub checksum: u8,                          // 7F: Checksum sum(0..127) == 0
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DetailedTiming {
    pub pixel_clock: u16, // Unit 10kHz
    pub h_active_low: u8,
    pub h_blank_low: u8,
    pub h_active_blank_high: u8,
    pub v_active_low: u8,
    pub v_blank_low: u8,
    pub v_active_blank_high: u8,
    pub h_sync_offset_low: u8,
    pub h_sync_width_low: u8,
    pub v_sync_low: u8, // offset and width
    pub sync_high: u8,  // h/v sync offset/width high bits
    pub h_size_low: u8,
    pub v_size_low: u8,
    pub h_v_size_high: u8,
    pub h_border: u8,
    pub v_border: u8,
    pub flags: u8,
}
