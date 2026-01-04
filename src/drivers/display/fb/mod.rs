//! # Framebuffer Core
//!
//! Gerenciamento genérico de framebuffers. Funciona com qualquer
//! fonte de framebuffer (GOP, VBE, hardware GPU).
//!
//! ## Componentes:
//! - **BufferManager**: Alocação de buffers de display
//! - **CRTC**: Controlador de saída física

pub mod buffer;
pub mod crtc;

use crate::core::boot::handoff::FramebufferInfo as HandoffFbInfo;

pub use buffer::{BufferManager, DisplayBuffer, BUFFER_MANAGER};
pub use crtc::{Crtc, DISPLAY_CRTC};

/// Inicializa a infraestrutura de framebuffer.
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(Display/FB) Inicializando framebuffer...");

    // Inicializa CRTC com framebuffer do bootloader
    crtc::init(info);

    crate::kinfo!("(Display/FB) Framebuffer operacional");
}

/// Permite que um driver substitua o CRTC legado.
pub fn register_hardware_crtc(new_crtc: Crtc) {
    let mut current = DISPLAY_CRTC.lock();
    crate::kinfo!("(Display/FB) Migrando para hardware nativo");
    *current = new_crtc;
}
