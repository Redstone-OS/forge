//! Video Drivers

pub mod font;
pub mod font_data;
pub mod framebuffer;

pub use framebuffer::init as init_fb;

/// Escreve bytes no console (framebuffer)
pub fn console_write_bytes(_bytes: &[u8]) {
    // Stub: Implementar renderização de fonte
}
