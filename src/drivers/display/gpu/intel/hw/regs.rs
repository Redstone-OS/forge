//! # Intel GPU Register Definitions
//!
//! Definições de registros MMIO do GPU Intel.
//! Baseado na documentação PRM (Programmer's Reference Manual) da Intel.
//!
//! ## Organização
//!
//! Os registros são organizados por funcionalidade:
//! - Display Engine (pipes, planes, transcoders)
//! - Graphics Translation Table (GTT)
//! - Power Management
//! - Interrupt Control

#![allow(dead_code)]

// =============================================================================
// DISPLAY ENGINE - PIPE CONFIGURATION
// =============================================================================

/// Pipe A Configuration
pub const PIPEACONF: u32 = 0x70008;
/// Pipe B Configuration  
pub const PIPEBCONF: u32 = 0x71008;
/// Pipe C Configuration
pub const PIPECCONF: u32 = 0x72008;

/// Pipe enable bit
pub const PIPE_ENABLE: u32 = 1 << 31;
/// Pipe state (read-only)
pub const PIPE_STATE: u32 = 1 << 30;

/// Pipe A Source Size (resolution)
pub const PIPEASRC: u32 = 0x6001C;
/// Pipe B Source Size
pub const PIPEBSRC: u32 = 0x6101C;
/// Pipe C Source Size
pub const PIPECSRC: u32 = 0x6201C;

// =============================================================================
// DISPLAY ENGINE - PRIMARY PLANE
// =============================================================================

/// Primary Plane A Control
pub const DSPACNTR: u32 = 0x70180;
/// Primary Plane A Linear Offset
pub const DSPALINOFF: u32 = 0x70184;
/// Primary Plane A Stride
pub const DSPASTRIDE: u32 = 0x70188;
/// Primary Plane A Surface Address
pub const DSPASURF: u32 = 0x7019C;
/// Primary Plane A Tile Offset
pub const DSPATILEOFF: u32 = 0x701A4;

/// Primary Plane B Control
pub const DSPBCNTR: u32 = 0x71180;
/// Primary Plane B Surface Address
pub const DSPBSURF: u32 = 0x7119C;

/// Plane enable bit
pub const PLANE_ENABLE: u32 = 1 << 31;

/// Pixel formats for primary plane
pub const PLANE_FORMAT_BGRX8888: u32 = 0b0110 << 26;
pub const PLANE_FORMAT_RGBX8888: u32 = 0b1110 << 26;
pub const PLANE_FORMAT_XRGB8888: u32 = 0b0110 << 26;

// =============================================================================
// DISPLAY ENGINE - CURSOR PLANE
// =============================================================================

/// Cursor A Control
pub const CURACNTR: u32 = 0x70080;
/// Cursor A Base Address
pub const CURABASE: u32 = 0x70084;
/// Cursor A Position
pub const CURAPOS: u32 = 0x70088;

// =============================================================================
// GTT - GRAPHICS TRANSLATION TABLE
// =============================================================================

/// GTT Page Table Entries start
/// Localizado no final do MMIO space (BAR0)
pub const GTT_PTE_BASE: u32 = 0x800000;

/// GTT entry present bit
pub const GTT_ENTRY_VALID: u64 = 1 << 0;
/// GTT entry local memory bit (Gen12+)
pub const GTT_ENTRY_LOCAL: u64 = 1 << 1;

/// GGTT Total Size register (Gen9+)
pub const GGC: u32 = 0x50;

// =============================================================================
// POWER MANAGEMENT
// =============================================================================

/// Forcewake register (Gen9+)
pub const FORCEWAKE_MT: u32 = 0xA188;
/// Forcewake ACK register
pub const FORCEWAKE_MT_ACK: u32 = 0x130044;

/// Forcewake kernel bit
pub const FORCEWAKE_KERNEL: u32 = 1 << 0;

/// Power well control
pub const PWR_WELL_CTL2: u32 = 0x45404;

// =============================================================================
// INTERRUPT CONTROL
// =============================================================================

/// Master Interrupt Control
pub const GEN8_MASTER_IRQ: u32 = 0x44200;

/// Display Engine Interrupt Enable
pub const DEIER: u32 = 0x4400C;
/// Display Engine Interrupt Identity
pub const DEIIR: u32 = 0x44008;

// =============================================================================
// DISPLAY TIMING
// =============================================================================

/// Horizontal Total A
pub const HTOTAL_A: u32 = 0x60000;
/// Horizontal Blank A
pub const HBLANK_A: u32 = 0x60004;
/// Horizontal Sync A
pub const HSYNC_A: u32 = 0x60008;
/// Vertical Total A
pub const VTOTAL_A: u32 = 0x6000C;
/// Vertical Blank A
pub const VBLANK_A: u32 = 0x60010;
/// Vertical Sync A
pub const VSYNC_A: u32 = 0x60014;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Calcula offset do pipe baseado no índice.
#[inline]
pub const fn pipe_offset(pipe: u8) -> u32 {
    (pipe as u32) * 0x1000
}

/// Calcula endereço de registro para pipe específico.
#[inline]
pub const fn pipe_reg(base_reg: u32, pipe: u8) -> u32 {
    base_reg + pipe_offset(pipe)
}
