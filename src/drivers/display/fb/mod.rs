//! # Framebuffer and KMS Subsystem (Kernel Mode Setting)
//!
//! Este módulo atua como o núcleo gráfico do RedstoneOS. Ele abstrai o hardware
//! de vídeo (gerenciado pelos drivers em `display/gpu`) e fornece uma interface
//! unificada para o sistema de janelas e consoles.
//!
//! ## Componentes:
//! - **BufferManager**: Gerencia a alocação de memória de vídeo (VRAM ou System RAM).
//! - **CRTC (Cathode Ray Tube Controller)**: Abstração da saída física (HDMI, DP, eDP).
//! - **Plane**: Camadas de composição (Hardware overlays) - *Planejado*.

pub mod buffer;
pub mod crtc;

use crate::core::boot::handoff::FramebufferInfo as HandoffFbInfo;
pub use buffer::{BufferManager, DisplayBuffer, BUFFER_MANAGER};
pub use crtc::{Crtc, DISPLAY_CRTC};

/// Inicializa a infraestrutura básica de display.
/// Esta função prepara os gerenciadores antes que os drivers de GPU sejam carregados.
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(Display/FB) Inicializando Núcleo de Gerenciamento Gráfico...");

    // 1. Inicializa o CRTC Primário com o buffer legado do bootloader (Fallback)
    // Isso garante que o kernel tenha saída visual mesmo se nenhum driver de GPU carregar.
    crtc::init(info);

    crate::kinfo!("(Display/FB) Infraestrutura KMS operacional.");
}

/// Permite que um driver de GPU (Intel, NVIDIA, etc.) substitua o CRTC legado
/// por um controlador de alto desempenho durante o boot.
pub fn register_hardware_crtc(new_crtc: Crtc) {
    let mut current = DISPLAY_CRTC.lock();
    crate::kinfo!("(Display/FB) Migrando de Framebuffer Legado para Hardware Nativo.");
    *current = new_crtc;
}
