//! Subsistema de Vídeo.
//!
//! Responsável por primitivas gráficas, gerenciamento de cores e fontes.
//! O Console usa este módulo para renderizar texto.

pub mod font;
pub mod font_data;
pub mod framebuffer;

use crate::core::handoff::{FramebufferInfo};
use crate::mm::vmm;

/// Informações globais do Framebuffer ativo.
static mut FRAMEBUFFER: Option<FramebufferInfo> = None;

/// Inicializa o driver de vídeo.
///
/// Mapeia a memória do framebuffer (se necessário) e limpa a tela.
pub unsafe fn init(info: &FramebufferInfo) {
    FRAMEBUFFER = Some(*info);
    crate::kinfo!(
        "Video Driver: {}x{} stride={} format={:?}",
        info.width,
        info.height,
        info.stride,
        info.format
    );

    // Mapear Framebuffer (Identity Map para simplicidade no kernel, assumindo endereço físico acessível)
    // Se o FB estiver acima de 4GB, precisamos garantir mapeamento.
    // O identity map inicial cobre 0-4GB.
    // Vamos garantir mapeamento explícito página por página.
    let start_page = info.addr & !0xFFF;
    let end_addr = info.addr + info.size;
    let end_page = (end_addr + 0xFFF) & !0xFFF;

    let mut curr = start_page;
    while curr < end_page {
        // Mapeia 1:1, RW, Kernel-only
        vmm::map_page(curr, curr, vmm::PAGE_PRESENT | vmm::PAGE_WRITABLE);
        // Otimização: Huge pages seria melhor, mas requer suporte no VMM map_page
        // ou map_range. Por enquanto, 4KB é seguro.
        curr += 4096;
    }

    // Limpar tela (Azul Escuro para teste)
    clear_screen(0x000F00);
}

/// Limpa a tela com uma cor sólida (formato 0x00RRGGBB).
pub fn clear_screen(color: u32) {
    unsafe {
        if let Some(fb) = FRAMEBUFFER {
            // Assume 32bpp (4 bytes por pixel)
            // TODO: Suportar outros formatos baseados em fb.format
            let ptr = fb.addr as *mut u32;
            let total_pixels = (fb.stride * fb.height) as usize; // Stride é largura em pixels (com padding)

            // Loop simples de preenchimento (pode ser otimizado com rep stosd)
            for i in 0..total_pixels {
                ptr.add(i).write_volatile(color);
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
