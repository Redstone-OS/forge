//! # Gen9 LP Specifics (Apollo Lake / Gemini Lake)
//!
//! Configurações específicas para GPUs Intel Gen9 LP:
//! - Apollo Lake (Celeron N3350, Pentium N4200)
//! - Gemini Lake (Celeron N4000, Pentium N5000)
//!
//! ## Características Gen9 LP
//!
//! - 3 Subslices, cada um com 8 Execution Units
//! - GTT de 4GB (48-bit addressing)
//! - Display Engine com 3 pipes
//! - Suporte a HDMI 2.0, DP 1.2

#![allow(dead_code)]

// =============================================================================
// DEVICE IDS
// =============================================================================

/// Device IDs para GPUs Gen9 LP (Apollo Lake)
pub const DEVICE_IDS_APL: &[u16] = &[
    0x5A84, // HD Graphics 505 (Pentium N4200)
    0x5A85, // HD Graphics 500 (Celeron N3350)
];

/// Device IDs para GPUs Gen9.5 (Gemini Lake)
pub const DEVICE_IDS_GLK: &[u16] = &[
    0x3184, // UHD Graphics 600
    0x3185, // UHD Graphics 605
];

// =============================================================================
// MEMORY CONFIGURATION
// =============================================================================

/// Tamanho máximo do GTT (Gen9 LP)
pub const MAX_GGTT_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4GB

/// Tamanho de uma entrada GTT
pub const GTT_ENTRY_SIZE: usize = 8; // 64 bits

/// Tamanho de página GTT
pub const GTT_PAGE_SIZE: usize = 4096;

// =============================================================================
// DISPLAY CONFIGURATION
// =============================================================================

/// Número máximo de pipes (Gen9 LP)
pub const MAX_PIPES: usize = 3;

/// Número máximo de planes por pipe
pub const MAX_PLANES_PER_PIPE: usize = 3; // Primary + 2 Sprites

/// Resolução máxima suportada
pub const MAX_WIDTH: u32 = 4096;
pub const MAX_HEIGHT: u32 = 4096;

// =============================================================================
// WORKAROUNDS
// =============================================================================

/// Gen9 LP requer WaForceWakeup antes de acessar certos registros
pub const WA_FORCE_WAKEUP_REQUIRED: bool = true;

/// Gen9 LP tem bug em page flip async
pub const WA_NO_ASYNC_FLIP: bool = false;

// =============================================================================
// FUNÇÕES DE DETECÇÃO
// =============================================================================

/// Verifica se um device ID é Gen9 LP.
pub fn is_gen9_lp(device_id: u16) -> bool {
    DEVICE_IDS_APL.contains(&device_id) || DEVICE_IDS_GLK.contains(&device_id)
}

/// Retorna nome legível do dispositivo.
pub fn device_name(device_id: u16) -> &'static str {
    match device_id {
        0x5A84 => "HD Graphics 505 (Apollo Lake)",
        0x5A85 => "HD Graphics 500 (Apollo Lake)",
        0x3184 => "UHD Graphics 600 (Gemini Lake)",
        0x3185 => "UHD Graphics 605 (Gemini Lake)",
        _ => "Unknown Intel GPU",
    }
}

/// Retorna geração do GPU.
pub fn gpu_generation(device_id: u16) -> u8 {
    if DEVICE_IDS_APL.contains(&device_id) {
        9 // Gen9 LP
    } else if DEVICE_IDS_GLK.contains(&device_id) {
        9 // Gen9.5 (reportado como 9)
    } else {
        0
    }
}
