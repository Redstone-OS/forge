//! # Teste do Compositor Gráfico
//!
//! Função temporária para testar o framebuffer sem userspace.

use crate::drivers::video::framebuffer as fb;

/// Cor laranja do Redstone
const ORANGE: u32 = 0xFF8C00;

/// Loop gráfico de teste (roda em kernel space)
/// Desenha fundo laranja para testar framebuffer
pub fn test_graphics_loop() -> ! {
    crate::kinfo!("(GFX) Iniciando teste gráfico...");

    // Obter info do framebuffer
    let info = match fb::get_info() {
        Some(i) => i,
        None => {
            crate::kerror!("(GFX) Framebuffer não disponível!");
            loop {
                crate::arch::Cpu::halt();
            }
        }
    };

    crate::kinfo!("(GFX) FB: ", info.width as u64);
    crate::ktrace!("(GFX) x", info.height as u64);

    // Limpar tela com cor laranja
    fb::clear(ORANGE);
    crate::kinfo!("(GFX) Tela limpa com cor laranja!");

    // Desenhar um retângulo branco no centro como teste
    let center_x = info.width / 2 - 50;
    let center_y = info.height / 2 - 25;
    fb::fill_rect(center_x, center_y, 100, 50, 0xFFFFFF);

    crate::kinfo!("(GFX) Retângulo de teste desenhado!");

    // Loop infinito
    loop {
        crate::arch::Cpu::halt();
    }
}
