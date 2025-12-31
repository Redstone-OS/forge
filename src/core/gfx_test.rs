//! # Teste do Compositor Gráfico
//!
//! Função temporária para testar o framebuffer sem userspace.

use crate::drivers::display::DISPLAY_CRTC;

/// Cor laranja do Redstone
const ORANGE: u32 = 0xFF8C00;

/// Loop gráfico de teste (roda em kernel space)
/// Desenha fundo laranja para testar framebuffer
pub fn test_graphics_loop() -> ! {
    crate::kinfo!("(GFX) Iniciando teste gráfico...");

    // Obter info do display
    let crtc = DISPLAY_CRTC.lock();
    let info = crtc.get_info();

    crate::kinfo!("(GFX) FB: ", info.width as u64);
    crate::ktrace!("(GFX) x", info.height as u64);

    // Limpar tela com cor laranja
    crtc.clear(ORANGE);
    crate::kinfo!("(GFX) Tela limpa com cor laranja!");

    drop(crtc); // Release lock

    crate::kinfo!("(GFX) Teste básico concluído!");

    // Loop infinito
    loop {
        crate::arch::Cpu::halt();
    }
}
