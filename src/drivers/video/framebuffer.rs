//! # Framebuffer Console
//!
//! Console de texto usando framebuffer.

use super::font;

/// Cores padrão
pub const BLACK: u32 = 0x000000;
pub const WHITE: u32 = 0xFFFFFF;
pub const BSOD_BLUE: u32 = 0x0000AA;

/// Posição do cursor (em pixels)
static mut CURSOR_X: u32 = 0;
static mut CURSOR_Y: u32 = 0;

/// Cor do texto
static mut TEXT_COLOR: u32 = WHITE;

/// Cor de fundo
static mut BG_COLOR: u32 = BLACK;

/// Constantes da fonte
pub const CHAR_WIDTH: u32 = 8;
pub const CHAR_HEIGHT: u32 = 16;

/// Escreve uma string no framebuffer.
pub fn console_write(fb_addr: u64, stride: u32, width: u32, height: u32, s: &str) {
    unsafe {
        if fb_addr == 0 {
            return;
        }

        let max_x = width.saturating_sub(CHAR_WIDTH);
        let max_y = height.saturating_sub(CHAR_HEIGHT);

        for c in s.bytes() {
            match c {
                b'\n' => {
                    CURSOR_X = 0;
                    CURSOR_Y += CHAR_HEIGHT;
                    if CURSOR_Y > max_y {
                        // Voltar ao topo (scroll simples)
                        CURSOR_Y = 0;
                    }
                }
                b'\r' => {
                    CURSOR_X = 0;
                }
                c => {
                    font::draw_char_raw(
                        fb_addr,
                        stride,
                        CURSOR_X as usize,
                        CURSOR_Y as usize,
                        c as char,
                        TEXT_COLOR,
                    );
                    CURSOR_X += CHAR_WIDTH;
                    if CURSOR_X > max_x {
                        CURSOR_X = 0;
                        CURSOR_Y += CHAR_HEIGHT;
                        if CURSOR_Y > max_y {
                            CURSOR_Y = 0;
                        }
                    }
                }
            }
        }
    }
}

/// Escreve bytes no framebuffer.
pub fn console_write_bytes(fb_addr: u64, stride: u32, width: u32, height: u32, buf: &[u8]) {
    unsafe {
        if fb_addr == 0 {
            return;
        }

        let max_x = width.saturating_sub(CHAR_WIDTH);
        let max_y = height.saturating_sub(CHAR_HEIGHT);

        for &c in buf {
            match c {
                b'\n' => {
                    CURSOR_X = 0;
                    CURSOR_Y += CHAR_HEIGHT;
                    if CURSOR_Y > max_y {
                        CURSOR_Y = 0;
                    }
                }
                b'\r' => {
                    CURSOR_X = 0;
                }
                c => {
                    font::draw_char_raw(
                        fb_addr,
                        stride,
                        CURSOR_X as usize,
                        CURSOR_Y as usize,
                        c as char,
                        TEXT_COLOR,
                    );
                    CURSOR_X += CHAR_WIDTH;
                    if CURSOR_X > max_x {
                        CURSOR_X = 0;
                        CURSOR_Y += CHAR_HEIGHT;
                        if CURSOR_Y > max_y {
                            CURSOR_Y = 0;
                        }
                    }
                }
            }
        }
    }
}

/// Define cor do texto.
pub fn set_text_color(color: u32) {
    unsafe {
        TEXT_COLOR = color;
    }
}

/// Define cor de fundo.
pub fn set_bg_color(color: u32) {
    unsafe {
        BG_COLOR = color;
    }
}

/// Reseta cursor.
pub fn reset_cursor() {
    unsafe {
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

/// Obtém posição do cursor.
pub fn get_cursor() -> (u32, u32) {
    unsafe { (CURSOR_X, CURSOR_Y) }
}
