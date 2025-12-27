//! # Video Driver - Minimal Framebuffer
//!
//! Driver de vídeo mínimo para suportar:
//! - Inicialização do framebuffer (GOP)
//! - Limpar tela com cor sólida
//! - Console de texto com fonte bitmap
//!
//! PID1 (userspace) é responsável por criar interface gráfica avançada.

pub mod font;
pub mod font_data;
pub mod framebuffer;

use crate::core::handoff::FramebufferInfo;

/// Informações globais do Framebuffer ativo.
static mut FRAMEBUFFER: Option<FramebufferInfo> = None;

/// Inicializa o driver de vídeo.
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
pub fn clear_screen(color: u32) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            let ptr = fb.addr as *mut u32;
            let total_pixels = (fb.stride * fb.height) as usize;

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

/// Exibe tela de panic (Blue Screen of Death).
///
/// Chamado pelo panic handler quando ocorre erro fatal.
pub fn panic_screen() {
    clear_screen(framebuffer::BSOD_BLUE);
    framebuffer::reset_cursor();
    framebuffer::set_text_color(framebuffer::WHITE);
}

// ============================================================================
// CONSOLE API (delega para framebuffer)
// ============================================================================

/// Escreve string no console.
pub fn console_write(s: &str) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            framebuffer::console_write(fb.addr, fb.stride, fb.width, fb.height, s);
        }
    }
}

/// Escreve bytes no console.
pub fn console_write_bytes(buf: &[u8]) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            framebuffer::console_write_bytes(fb.addr, fb.stride, fb.width, fb.height, buf);
        }
    }
}

/// Limpa console e reseta cursor.
pub fn console_clear() {
    clear_screen(0x000000);
    framebuffer::reset_cursor();
}

/// Define cor do texto.
pub fn set_text_color(color: u32) {
    framebuffer::set_text_color(color);
}

/// Define cor de fundo.
pub fn set_bg_color(color: u32) {
    framebuffer::set_bg_color(color);
}
