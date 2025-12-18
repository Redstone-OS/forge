//! /dev/fb* - Framebuffer (Gráfico)
//!
//! TODO(prioridade=baixa, versão=v2.0): Implementar framebuffer
//!
//! # Arquitetura Híbrida
//! - **Kernel:** Mapeia memória de vídeo, configura modo
//! - **Userspace:** Desenha pixels, renderiza UI
//!
//! # Implementação Sugerida
//! - /dev/fb0: Framebuffer principal
//! - mmap() para acesso direto à VRAM
//! - ioctl() para configurar resolução/profundidade
//! - Integrar com driver VGA/VESA/GOP

// TODO: Implementar FbDevice
