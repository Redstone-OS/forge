//! Text Console
//!
//! Provides a text-mode console using the framebuffer and font rendering.
//! Handles text output, scrolling, and cursor management.

use super::font::{draw_char, FONT_HEIGHT, FONT_WIDTH};
use super::framebuffer::{Color, Framebuffer, COLOR_BLACK, COLOR_WHITE};
use core::fmt;

/// Text console
pub struct Console {
    fb: Framebuffer,
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    fg_color: Color,
    bg_color: Color,
}

impl Console {
    /// Create a new console
    ///
    /// # Arguments
    /// * `fb` - Framebuffer to use for display
    pub fn new(fb: Framebuffer) -> Self {
        let cols = fb.width() / FONT_WIDTH;
        let rows = fb.height() / FONT_HEIGHT;

        Self {
            fb,
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: COLOR_WHITE,
            bg_color: COLOR_BLACK,
        }
    }

    /// Clear the console
    pub fn clear(&mut self) {
        self.fb.clear(self.bg_color);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Set foreground and background colors
    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    /// Write a single character
    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.cursor_x = 0,
            '\t' => {
                // Tab = 4 spaces
                for _ in 0..4 {
                    self.write_char(' ');
                }
            }
            c => {
                if self.cursor_x >= self.cols {
                    self.newline();
                }

                let x = self.cursor_x * FONT_WIDTH;
                let y = self.cursor_y * FONT_HEIGHT;

                draw_char(&mut self.fb, x, y, c, self.fg_color, self.bg_color);

                self.cursor_x += 1;
            }
        }
    }

    /// Write a string
    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    /// Move to next line
    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y >= self.rows {
            self.scroll();
        }
    }

    /// Scroll the console up by one line
    fn scroll(&mut self) {
        // Copy all lines up by one
        self.fb
            .copy_region(FONT_HEIGHT, 0, (self.rows - 1) * FONT_HEIGHT);

        // Clear the last line
        self.fb.fill_rect(
            0,
            (self.rows - 1) * FONT_HEIGHT,
            self.fb.width(),
            FONT_HEIGHT,
            self.bg_color,
        );

        self.cursor_y = self.rows - 1;
    }
}

/// Implement fmt::Write for Console
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
