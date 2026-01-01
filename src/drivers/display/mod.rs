//! # Display Driver Module
//!
//! Driver de display moderno inspirado em DRM/KMS.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Display Subsystem             │
//! ├─────────────┬─────────────┬─────────────┤
//! │   Buffer    │    CRTC     │   Plane     │
//! │   Manager   │  (Output)   │  (Future)   │
//! └─────────────┴─────────────┴─────────────┘
//! ```

pub mod buffer;
pub mod crtc;

use crate::core::boot::handoff::FramebufferInfo as HandoffFbInfo;

pub use buffer::{BufferManager, DisplayBuffer, BUFFER_MANAGER};
pub use crtc::{Crtc, DISPLAY_CRTC};

/// Inicializa o subsistema de display.
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(Display) Inicializando subsistema de display...");
    crate::ktrace!("(Display) Width:", info.width as u64);
    crate::ktrace!("(Display) Height:", info.height as u64);
    crate::ktrace!("(Display) Stride:", info.stride as u64);

    // Inicializar CRTC com informações do bootloader
    crtc::init(info);

    crate::kinfo!("(Display) Subsistema inicializado com sucesso!");
}
