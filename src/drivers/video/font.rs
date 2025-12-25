//! Renderizador de Fontes (Bitmap).
//!
//! Desenha caracteres na tela pixel a pixel.

use crate::drivers::video::font_data::FONT_8X16;

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
    // Criamos um slice grande o suficiente para cobrir o FB.
    let fb = unsafe { core::slice::from_raw_parts_mut(fb_addr as *mut u32, 4 * 1024 * 1024) };

    // Mapeia char para índice na fonte (ASCII 0x00..0xFF)
    // Nossa fonte hardcoded cobre ASCII 0..255 (Code Page 437 style).
    let idx = c as usize;
    if idx >= 256 {
        return; // Não suportamos Unicode ainda
    }

    let glyph_offset = idx * 16;
    let glyph = &FONT_8X16[glyph_offset..glyph_offset + 16];

    for dy in 0..16 {
        let line = glyph[dy];
        for dx in 0..8 {
            // No VGA, bit 7 é o pixel mais à esquerda (x)
            // bit 0 é o pixel mais à direita (x+7)
            let is_pixel_on = (line >> (7 - dx)) & 1 != 0;

            if is_pixel_on {
                let offset = (y + dy) * (stride as usize) + (x + dx);
                if offset < fb.len() {
                    fb[offset] = color;
                }
            }
        }
    }
}
