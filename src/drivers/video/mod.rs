//! # Video Driver - Minimal Framebuffer
//!
//! Driver de vídeo mínimo para suportar apenas:
//! - Inicialização do framebuffer (GOP)
//! - Limpar tela com cor sólida
//! - Desenhar pixels individuais
//!
//! ## Arquitetura
//!
//! O kernel NÃO implementa console gráfico ou terminal.
//! PID1 (userspace) é responsável por criar interface gráfica.
//!
//! Este driver existe apenas para:
//! 1. Mostrar tela de panic com cor sólida
//! 2. Fornecer framebuffer info para syscalls
//!
//! Drivers de GPU avançados (NVIDIA, AMD, Intel) são carregados como módulos.

pub mod font;
pub mod font_data;
pub mod framebuffer;

use crate::core::handoff::FramebufferInfo;

/// Informações globais do Framebuffer ativo.
static mut FRAMEBUFFER: Option<FramebufferInfo> = None;

/// Inicializa o driver de vídeo.
///
/// Mapeia a memória do framebuffer (se necessário) e limpa a tela.
pub unsafe fn init(info: &FramebufferInfo) {
    FRAMEBUFFER = Some(*info);

    // Mapear framebuffer na memória virtual
    let start_page = info.addr & !0xFFF;
    let end_addr = info.addr + info.size;
    let end_page = (end_addr + 0xFFF) & !0xFFF;

    let mut curr = start_page;
    while curr < end_page {
        let _ = crate::mm::vmm::map_page(
            curr,
            curr,
            crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE,
        );
        curr += 4096;
    }

    // Limpar tela com cor escura
    clear_screen(0x000000);
}

/// Limpa a tela com uma cor sólida (formato 0x00RRGGBB).
///
/// # Implementação
/// Usa loop manual + write_volatile para evitar otimizações SSE.
pub fn clear_screen(color: u32) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            let ptr = fb.addr as *mut u32;
            let total_pixels = (fb.stride * fb.height) as usize;

            // Loop manual sem SIMD
            let mut i = 0usize;
            while i < total_pixels {
                ptr.add(i).write_volatile(color);
                i += 1;
            }
        }
    }
}

/// Desenha um pixel na tela.
pub fn put_pixel(x: u32, y: u32, color: u32) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            if x >= fb.width || y >= fb.height {
                return;
            }
            let offset = (y * fb.stride + x) as usize;
            let ptr = fb.addr as *mut u32;
            ptr.add(offset).write_volatile(color);
        }
    }
}

/// Retorna informações do framebuffer (para syscalls).
pub fn get_info() -> Option<FramebufferInfo> {
    unsafe { FRAMEBUFFER }
}

/// Exibe tela de panic (vermelho sólido).
///
/// Chamado pelo panic handler quando ocorre erro fatal.
pub fn panic_screen() {
    clear_screen(0x800000); // Vermelho escuro
}
