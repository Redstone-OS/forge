//! # Video Subsystem (Framebuffer)
//!
//! O subsistema de vídeo é responsável por gerenciar a memória de vídeo (LFB - Linear Framebuffer)
//! entregue pelo bootloader via GOP (Graphics Output Protocol).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Gerenciamento de Memória de Vídeo:** Mapeia a região física do framebuffer para o espaço virtual do kernel.
//! - **Primitivas Gráficas:** Fornece funções de baixo nível (`put_pixel`, `clear_screen`) usadas por consumidores como o Console.
//! - **Abstração de Formato:** Deve (futuramente) lidar com conversão de formatos de pixel (RGB, BGR, etc).
//!
//! ## 🏗️ Arquitetura Atual
//! | Componente    | Função | Status |
//! |---------------|--------|--------|
//! | `framebuffer` | Structs e definições do layout de memória. | **Passivo:** Apenas dados. |
//! | `font`        | Renderizador de bitmap fonts (Fixed Width). | **Básico:** Renderiza glifos byte-a-byte. |
//! | `mod.rs`      | Glue logic e funções globais (`init`, `put_pixel`). | **Unsafe Global:** Usa `static mut` sem VRAM lock adequado. |
//!
//! ## 🔍 Análise Crítica
//!
//! ### ✅ Pontos Fortes
//! - **Agnóstico de Hardware:** Funciona em qualquer GPU compatível com VESA/UEFI GOP.
//! - **Zero Alocação:** As primitivas de desenho não alocam memória no heap, seguro para uso em Panic/Exception handlers.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Performance (Software Rendering):** Toda operação gráfica é feita pela CPU escrevendo na VRAM. Sem aceleração de hardware.
//!   - *Gargalo:* Limpar a tela ou rolar o console em resoluções 4K é visivelmente lento.
//! - **Falta de Double Buffering:** Desenhamos direto na tela ("Front Buffer"). Isso causa "flickering" e "tearing".
//! - **Segurança de Memória:** O acesso ao `FRAMEBUFFER` estático é `unsafe` e não sincronizado. Duas cores tentando desenhar ao mesmo tempo causarão Data Race.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Performance)** Implementar *Dirty Rectangles* ou *Damage Tracking*.
//!   - *Motivo:* Redesenhar apenas o que mudou, em vez da tela toda.
//! - [ ] **TODO: (Architecture)** Criar uma abstração `Surface` ou `Canvas`.
//!   - *Motivo:* Permitir desenhar em buffers off-screen (Back Buffer) para implementar Double Buffering.
//! - [ ] **TODO: (Safety)** Encapsular o `FRAMEBUFFER` global em um `Spinlock<Framebuffer>`.
//!   - *Impacto:* Prevenir Data Races em ambientes Multicore.
//! - [ ] **TODO: (Feature)** Suporte a aceleração 2D básica (Blit).
//!   - *Nota:* Difícil sem drivers específicos de GPU, mas otimizações SIMD (AVX/SSE) para `memcpy` de vídeo ajudam.

pub mod font;
pub mod font_data;
pub mod framebuffer;

use crate::core::handoff::FramebufferInfo;
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
