//! Primitivas de Framebuffer.
//!
//! Define cores e operações básicas de pixel que independem de texto.

/// Definição de Cores (32-bit).
/// Layout genérico (0xAARRGGBB), o driver deve converter conforme o formato do hardware (RGB/BGR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Color(pub u32);

// Paleta VGA básica
pub const BLACK: Color = Color(0x000000);
pub const WHITE: Color = Color(0xFFFFFF);
pub const RED: Color = Color(0xFF0000);
pub const GREEN: Color = Color(0x00FF00);
pub const BLUE: Color = Color(0x0000FF);
pub const GREY: Color = Color(0x888888);
