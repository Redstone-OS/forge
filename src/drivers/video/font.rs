//! Renderizador de Fontes (Bitmap).
//!
//! Desenha caracteres na tela pixel a pixel.

// Fonte 8x8 Basic (Stub para compilação).
// Em produção, isso seria um array `static [u8; 4096]` com a fonte VGA ou PSF.
// Para este MVP, desenhamos um bloco sólido se o bitmask não estiver disponível.

/// Desenha um caractere na posição (x, y).
///
/// # Arguments
/// * `fb_addr`: Endereço linear do framebuffer.
/// * `stride`: Largura da linha em pixels.
/// * `x`, `y`: Coordenadas em pixels.
/// * `c`: Caractere a desenhar.
/// * `color`: Cor do texto (0xAARRGGBB).
pub fn draw_char_raw(fb_addr: u64, stride: u32, x: usize, y: usize, c: char, color: u32) {
    if fb_addr == 0 {
        return;
    }

    // Safety: O caller (Console) garante que fb_addr é válido e mapeado.
    // Criamos um slice grande o suficiente para não dar panic, mas o ideal é passar o tamanho real.
    let fb = unsafe { core::slice::from_raw_parts_mut(fb_addr as *mut u32, 4 * 1024 * 1024) };

    // Glifo 8x16 simples (hardcoded blocky font para debug visual)
    // Se c == ' ', não desenha nada (transparente/fundo).
    if c == ' ' {
        return;
    }

    for dy in 0..16 {
        for dx in 0..8 {
            let offset = (y + dy) * (stride as usize) + (x + dx);

            // Simulação de fonte: desenha um quadrado com borda
            // Em um OS real, aqui leríamos `FONT_DATA[c as usize][dy] & (1 << dx)`
            let is_pixel_on = dx > 0 && dx < 7 && dy > 0 && dy < 15;

            if is_pixel_on && offset < fb.len() {
                fb[offset] = color;
            }
        }
    }
}
