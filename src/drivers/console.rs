//! Driver de Console de Vídeo (Framebuffer).
//!
//! Gerencia a escrita de texto na tela gráfica.

use crate::core::handoff::{FramebufferInfo, PixelFormat};
use crate::drivers::video::font;
use core::fmt; // Assumindo que font.rs foi mantido/migrado para drivers/video/font.rs

pub struct Console {
    info: FramebufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl Console {
    pub fn new(info: FramebufferInfo) -> Self {
        Self {
            info,
            x_pos: 0,
            y_pos: 0,
        }
    }

    /// Limpa a tela (preenche com preto).
    pub fn clear(&mut self) {
        let size = self.info.size as usize;
        let buffer =
            unsafe { core::slice::from_raw_parts_mut(self.info.addr as *mut u32, size / 4) };
        buffer.fill(0x00000000);
        self.x_pos = 0;
        self.y_pos = 0;
    }

    fn newline(&mut self) {
        self.x_pos = 0;
        self.y_pos += 16; // Font Height

        // Simples wrap-around por enquanto (scroll é complexo para implementar agora)
        if self.y_pos >= self.info.height as usize {
            self.y_pos = 0;
            self.clear();
        }
    }

    fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        if self.x_pos + 8 >= self.info.width as usize {
            self.newline();
        }

        // Desenha o caractere (abstração simples)
        // Nota: Assumimos BGR/RGB 32bit.
        font::draw_char_raw(
            self.info.addr,
            self.info.stride,
            self.x_pos,
            self.y_pos,
            c,
            0xFFFFFFFF,
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
