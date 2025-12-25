//! Driver de Console de Vídeo.
//!
//! Gerencia a escrita de texto na tela gráfica (Framebuffer).
//! Suporta cores, quebras de linha e rolagem de tela (Scroll).

use crate::core::handoff::FramebufferInfo;
use core::fmt;
// Importação corrigida: depende de src/drivers/video/mod.rs exportar 'pub mod font;'
use crate::drivers::video::font;
use crate::sync::Mutex;

/// Driver Global de Console.
///
/// Protegido por um Mutex (Spinlock) para garantir acesso seguro em ambiente multicore/interrupção.
/// O Option permite que o console seja inicializado tardiamente (após handoff de vídeo).
pub static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

/// Inicializa o console de vídeo.
///
/// # Argumentos
/// * `info`: Informações do Framebuffer obtidas do BootInfo.
pub fn init_console(info: FramebufferInfo) {
    let mut console_lock = CONSOLE.lock();
    let mut console = Console::new(info);
    console.clear();
    *console_lock = Some(console);
}

/// Helper para escrever fmt::Arguments no console global (se inicializado).
pub fn console_print_fmt(args: fmt::Arguments) {
    if let Some(mut console) = CONSOLE.try_lock() {
        if let Some(ref mut c) = *console {
            let _ = fmt::write(c, args);
        }
    }
}

/// Definição de cores (32-bit ARGB/BGRA).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u32);

impl Color {
    pub const fn new(val: u32) -> Self {
        Self(val)
    }
}

// Cores padrão para uso no kernel (Restauradas para o main.rs funcionar)
pub const COLOR_BLACK: Color = Color(0x000000);
pub const COLOR_WHITE: Color = Color(0xFFFFFF);
pub const COLOR_RED: Color = Color(0xFF0000);
pub const COLOR_GREEN: Color = Color(0x00FF00);
pub const COLOR_BLUE: Color = Color(0x0000FF);
pub const COLOR_LIGHT_GREEN: Color = Color(0x00FF00);

pub struct Console {
    info: FramebufferInfo,
    x_pos: usize,
    y_pos: usize,
    fg_color: u32,
    bg_color: u32,
}

impl Console {
    /// Cria uma nova instância do Console.
    pub fn new(info: FramebufferInfo) -> Self {
        Self {
            info,
            x_pos: 0,
            y_pos: 0,
            fg_color: 0xFFFFFF, // Branco padrão
            bg_color: 0x000000, // Preto padrão
        }
    }

    /// Define as cores de frente e fundo.
    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg_color = fg.0;
        self.bg_color = bg.0;
    }

    /// Limpa a tela preenchendo com a cor de fundo.
    pub fn clear(&mut self) {
        // Assume 32bpp (4 bytes por pixel)
        let size_u32 = self.info.size as usize / 4;
        let buffer =
            unsafe { core::slice::from_raw_parts_mut(self.info.addr as *mut u32, size_u32) };
        buffer.fill(self.bg_color);
        self.x_pos = 0;
        self.y_pos = 0;
    }

    /// Avança para a próxima linha, rolando a tela se necessário.
    fn newline(&mut self) {
        self.x_pos = 0;
        self.y_pos += 16; // Altura da fonte (8x16)

        // Se passarmos da altura da tela, rolar o conteúdo para cima
        if self.y_pos + 16 > self.info.height as usize {
            self.scroll();
            self.y_pos -= 16;
        }
    }

    /// Rola a tela para cima (Move memória de vídeo).
    fn scroll(&mut self) {
        // DEBUG: Trace scroll para achar GPF
        {
            use core::fmt::Write;
            if let Some(mut serial) = crate::drivers::serial::SERIAL1.try_lock() {
                let _ = write!(serial, "[DEBUG] Console::scroll start\n");
            }
        }

        let stride = self.info.stride as usize;
        let height = self.info.height as usize;

        // Copiar linhas de baixo para cima
        // Fonte 8x16: copiar (height - 16) linhas
        // Multiplica por 4 bytes (u32) implicitamente ao trabalhar com slice u32
        let u32_stride = stride;
        let font_height = 16;

        if height < font_height {
            return;
        } // Proteção

        let lines_to_copy = height - font_height;
        let u32s_to_copy = lines_to_copy * u32_stride;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                self.info.addr as *mut u32,
                (self.info.size / 4) as usize,
            )
        };

        // Validação de bounds exata
        let total_pixels = stride * height;
        if buffer.len() < total_pixels {
            // Panic seguro (serial only)
            use core::fmt::Write;
            if let Some(mut serial) = crate::drivers::serial::SERIAL1.try_lock() {
                let _ = write!(
                    serial,
                    "[PANIC] Framebuffer menor que stride*height! Len={} Req={}\n",
                    buffer.len(),
                    total_pixels
                );
            }
            return;
        }

        // Mover o conteúdo da tela para cima.
        // copy_within é seguro para sobreposição (memmove).
        // Copia do início da segunda linha (stride * 16) até o final desejado, para o início (0).
        let start_src = u32_stride * font_height;
        let end_src = start_src + u32s_to_copy;

        // Bounds Check Paranoico
        if end_src > buffer.len() {
            use core::fmt::Write;
            if let Some(mut serial) = crate::drivers::serial::SERIAL1.try_lock() {
                let _ = write!(
                    serial,
                    "[PANIC] Scroll Overflow! EndSrc={} Buffer={}\n",
                    end_src,
                    buffer.len()
                );
            }
            return;
        }

        buffer.copy_within(start_src..end_src, 0);

        // Limpar a última linha (área nova em baixo)
        let start_clear = u32s_to_copy;
        let end_clear = height * u32_stride;

        if end_clear <= buffer.len() {
            buffer[start_clear..end_clear].fill(self.bg_color);
        }

        {
            use core::fmt::Write;
            if let Some(mut serial) = crate::drivers::serial::SERIAL1.try_lock() {
                let _ = write!(serial, "[DEBUG] Console::scroll end\n");
            }
        }
    }

    /// Escreve um caractere na posição atual.
    fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        // Wrap automático se passar da largura
        if self.x_pos + 8 > self.info.width as usize {
            self.newline();
        }

        // Desenhar caractere usando o módulo de fonte
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
