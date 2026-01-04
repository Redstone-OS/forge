//! # Subsistema de Display (Vídeo e Gráficos)
//!
//! Este módulo gerencia a saída visual do RedstoneOS, desde framebuffers simples
//! até drivers de aceleração 2D/3D.
//!
//! ## Estrutura:
//! - **fb/ (Framebuffer)**: Gerenciamento genérico de buffers de memória de vídeo.
//! - **bochs/**: Driver para o emulador Bochs e QEMU (BGA).
//! - **virtio/**: Driver VirtIO-GPU para virtualização de alto desempenho.
//! - **gpu/**: Implementações para hardware real (Intel, AMD, NVIDIA).
//! - **edid/**: Parser para informações de monitores e resoluções suportadas.

pub mod bochs;
pub mod edid;
pub mod fb;
pub mod gpu;
pub mod virtio;

pub use fb::{BUFFER_MANAGER, DISPLAY_CRTC};

use crate::core::boot::handoff::FramebufferInfo as HandoffFbInfo;

/// Inicializa os drivers de display disponíveis no sistema.
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(Display) Inicializando subsistema gráfico...");

    // 1. Inicializa o núcleo de gerenciamento de buffers e CRTC legado
    fb::init(info);

    // 2. Registra os drivers no RDM para pareamento automático com hardware real
    bochs::init();
    virtio::init();
    gpu::init();
}
