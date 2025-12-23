//! Driver de Console de Vídeo.
//!
//! Gerencia a escrita de texto na tela gráfica (Framebuffer).
//! Suporta cores, quebras de linha e rolagem simples.

use crate::core::handoff::FramebufferInfo;
use core::fmt;
// O PixelFormat pode ser útil se precisarmos converter cores, mas por enquanto usamos u32 direto ou Color.
// Se Color estiver em framebuffer.rs, precisamos importá-lo ou redefini-lo.
// Assumindo que definiremos cores simples aqui para evitar dependência circular complexa ou re-exportaremos.

use crate::drivers::video::font;

// Definição local de cores para o console (ou importar de framebuffer se lá estiver público)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u32);

pub const COLOR_BLACK: Color = Color(0x000000);
pub const COLOR_WHITE: Color = Color(0xFFFFFF);
pub const COLOR_RED: Color = Color(0xFF0000);
pub const COLOR_GREEN: Color = Color(0x00FF00);
pub const COLOR_BLUE: Color = Color(0x0000FF);
pub const COLOR_LIGHT_GREEN: Color = Color(0x00FF00); // Simplificação

pub struct Console {
    info: FramebufferInfo,
    x_pos: usize,
    y_pos: usize,
    fg_color: u32,
    bg_color: u32,
    cols: usize,
    rows: usize,
}

impl Console {
    pub fn new(info: FramebufferInfo) -> Self {
        // Assumindo fonte 8x16
        let cols = info.width as usize / 8;
        let rows = info.height as usize / 16;

        Self {
            info,
            x_pos: 0,
            y_pos: 0,
            fg_color: 0xFFFFFF, // Branco padrão
            bg_color: 0x000000, // Preto padrão
            cols,
            rows,
        }
    }

    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg_color = fg.0;
        self.bg_color = bg.0;
    }

    pub fn clear(&mut self) {
        // Preencher a tela com a cor de fundo
        // Nota: Assumimos 32bpp (4 bytes por pixel) para simplificar a escrita direta
        // Em um driver real, verificaríamos info.bytes_per_pixel
        let size_u32 = self.info.size as usize / 4;
        let buffer =
            unsafe { core::slice::from_raw_parts_mut(self.info.addr as *mut u32, size_u32) };
        buffer.fill(self.bg_color);
        self.x_pos = 0;
        self.y_pos = 0;
    }

    fn newline(&mut self) {
        self.x_pos = 0;
        self.y_pos += 16; // Altura da fonte

        // Scroll simples se atingir o fim da tela
        if self.y_pos + 16 > self.info.height as usize {
            self.scroll();
            self.y_pos -= 16;
        }
    }

    fn scroll(&mut self) {
        // Mover memória para cima
        let stride = self.info.stride as usize;
        let height = self.info.height as usize;
        let width = self.info.width as usize;

        // Copiar linhas de baixo para cima
        // Fonte 8x16: copiar (height - 16) linhas
        let lines_to_copy = height - 16;
        let dwords_to_copy = lines_to_copy * stride;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                self.info.addr as *mut u32,
                (self.info.size / 4) as usize,
            )
        };

        // Memmove manual ou copy_within se disponível
        // copy_within é seguro para sobreposição
        buffer.copy_within(stride * 16..(stride * 16) + dwords_to_copy, 0);

        // Limpar a última linha (área nova)
        let last_line_start = dwords_to_copy;
        let last_line_end = height * stride;
        // Preencher com bg_color, cuidado com limites
        if last_line_end <= buffer.len() {
            buffer[last_line_start..last_line_end].fill(self.bg_color);
        }
    }

    fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        // Wrap automático se passar da largura
        if self.x_pos + 8 > self.info.width as usize {
            self.newline();
        }

        // Desenhar caractere usando a função de fonte corrigida
        font::draw_char_raw(
            self.info.addr,
            self.info.stride,
            self.x_pos,
            self.y_pos,
            c,
            self.fg_color,
        );

        self.x_pos += 8;
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
