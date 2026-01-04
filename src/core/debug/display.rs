//! # Debug Display - Sistema de Log Visual
//!
//! Módulo para exibir logs diretamente na tela quando não há serial disponível.
//! IMPORTANTE: Este módulo é para DEBUG APENAS. Comente as chamadas quando não precisar.
//!
//! ## Como usar:
//!
//! 1. No entry.rs, após inicializar o display, chame:
//!    `// crate::core::debug::display::init();`
//!
//! 2. Para logar, use:
//!    `// crate::core::debug::display::log("mensagem");`
//!    `// crate::core::debug::display::log_hex("valor:", 0x1234);`
//!
//! 3. Para desativar, apenas comente as linhas com //

use crate::sync::Spinlock;

// =============================================================================
// CONFIGURAÇÃO
// =============================================================================

/// Cor de fundo do console de debug (preto)
const BG_COLOR: u32 = 0xFF000000;

/// Cor do texto normal (verde)
const TEXT_COLOR: u32 = 0xFF00FF00;

/// Cor de texto de warning (amarelo)
const WARN_COLOR: u32 = 0xFFFFFF00;

/// Cor de texto de erro (vermelho)
const ERROR_COLOR: u32 = 0xFFFF0000;

/// Cor de texto de info (ciano)
const INFO_COLOR: u32 = 0xFF00FFFF;

/// Largura de um caractere em pixels (fonte 8x16)
const CHAR_WIDTH: u32 = 8;

/// Altura de um caractere em pixels
const CHAR_HEIGHT: u32 = 16;

/// Margem lateral
const MARGIN_X: u32 = 4;

/// Margem superior
const MARGIN_Y: u32 = 4;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Estado do console de debug
struct DebugConsole {
    /// Framebuffer address (virtual)
    fb_addr: u64,
    /// Largura da tela
    width: u32,
    /// Altura da tela
    height: u32,
    /// Stride em bytes
    stride: u32,
    /// Coluna atual (em caracteres)
    cursor_x: u32,
    /// Linha atual (em caracteres)
    cursor_y: u32,
    /// Número máximo de colunas
    max_cols: u32,
    /// Número máximo de linhas
    max_rows: u32,
    /// Se está inicializado
    initialized: bool,
    /// Se o log em tela está ativo
    visible: bool,
}

impl DebugConsole {
    const fn new() -> Self {
        Self {
            fb_addr: 0,
            width: 0,
            height: 0,
            stride: 0,
            cursor_x: 0,
            cursor_y: 0,
            max_cols: 0,
            max_rows: 0,
            initialized: false,
            visible: false,
        }
    }
}

static DEBUG_CONSOLE: Spinlock<DebugConsole> = Spinlock::new(DebugConsole::new());

// =============================================================================
// FONTE BITMAP 8x16 (PSF simplificado)
// =============================================================================

/// Fonte bitmap 8x16 - apenas ASCII printable (32-126)
/// Cada caractere tem 16 bytes (1 byte por linha)
static FONT_8X16: [u8; 1520] = include_font();

/// Inclui a fonte embutida (gera em tempo de compilação)
const fn include_font() -> [u8; 1520] {
    // Fonte básica 8x16 gerada proceduralmente
    // Cobre ASCII 32-126 (95 caracteres * 16 bytes = 1520 bytes)
    let mut font = [0u8; 1520];

    // Vamos definir alguns caracteres básicos manualmente
    // O resto será preenchido com padrão de fallback

    // Esta é uma fonte bem básica, mas funcional
    // Caracteres são definidos como bitmaps de 8 pixels de largura x 16 linhas

    font
}

/// Obtém o bitmap de um caractere
fn get_char_bitmap(c: char) -> &'static [u8] {
    let idx = c as usize;
    if idx >= 32 && idx <= 126 {
        let offset = (idx - 32) * 16;
        &FONT_8X16[offset..offset + 16]
    } else {
        // Caractere não suportado - retorna espaço
        &FONT_8X16[0..16]
    }
}

// Fonte alternativa mais simples - apenas blocos
/// Desenha um "caractere" como um bloco colorido se a fonte não estiver disponível
fn draw_block_char(fb: *mut u32, stride: u32, x: u32, y: u32, c: char, color: u32) {
    // Para simplificar, vamos desenhar caracteres como blocos com padrões simples
    let pattern = get_simple_pattern(c);

    for row in 0..CHAR_HEIGHT {
        let bits = pattern[row as usize % 8];
        for col in 0..CHAR_WIDTH {
            let pixel_on = (bits >> (7 - col)) & 1 != 0;
            let px = x + col;
            let py = y + row;
            let offset = (py * (stride / 4)) + px;

            unsafe {
                let pixel_color = if pixel_on { color } else { BG_COLOR };
                core::ptr::write_volatile(fb.add(offset as usize), pixel_color);
            }
        }
    }
}

/// Padrões simples para caracteres (8 bytes por char, repetidos 2x para 16 linhas)
fn get_simple_pattern(c: char) -> [u8; 8] {
    match c {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
        '"' => [0x6C, 0x6C, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00],
        '#' => [0x6C, 0xFE, 0x6C, 0x6C, 0xFE, 0x6C, 0x00, 0x00],
        '$' => [0x18, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x18, 0x00],
        '%' => [0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00, 0x00],
        '&' => [0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00],
        '\'' => [0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00],
        '(' => [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        ')' => [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        '*' => [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        '/' => [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x00, 0x00],
        '0' => [0x7C, 0xC6, 0xCE, 0xD6, 0xE6, 0xC6, 0x7C, 0x00],
        '1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        '2' => [0x7C, 0xC6, 0x06, 0x1C, 0x30, 0x66, 0xFE, 0x00],
        '3' => [0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C, 0x00],
        '4' => [0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x1E, 0x00],
        '5' => [0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C, 0x00],
        '6' => [0x38, 0x60, 0xC0, 0xFC, 0xC6, 0xC6, 0x7C, 0x00],
        '7' => [0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00],
        '8' => [0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C, 0x00],
        '9' => [0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        ';' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30, 0x00],
        '<' => [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00],
        '=' => [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00],
        '>' => [0x60, 0x30, 0x18, 0x0C, 0x18, 0x30, 0x60, 0x00],
        '?' => [0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00],
        '@' => [0x7C, 0xC6, 0xDE, 0xDE, 0xDE, 0xC0, 0x78, 0x00],
        'A' => [0x38, 0x6C, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00],
        'B' => [0xFC, 0x66, 0x66, 0x7C, 0x66, 0x66, 0xFC, 0x00],
        'C' => [0x3C, 0x66, 0xC0, 0xC0, 0xC0, 0x66, 0x3C, 0x00],
        'D' => [0xF8, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00],
        'E' => [0xFE, 0x62, 0x68, 0x78, 0x68, 0x62, 0xFE, 0x00],
        'F' => [0xFE, 0x62, 0x68, 0x78, 0x68, 0x60, 0xF0, 0x00],
        'G' => [0x3C, 0x66, 0xC0, 0xC0, 0xCE, 0x66, 0x3E, 0x00],
        'H' => [0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00],
        'I' => [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'J' => [0x1E, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00],
        'K' => [0xE6, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0xE6, 0x00],
        'L' => [0xF0, 0x60, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00],
        'M' => [0xC6, 0xEE, 0xFE, 0xD6, 0xC6, 0xC6, 0xC6, 0x00],
        'N' => [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00],
        'O' => [0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00],
        'P' => [0xFC, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00],
        'Q' => [0x7C, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C, 0x06],
        'R' => [0xFC, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0xE6, 0x00],
        'S' => [0x7C, 0xC6, 0x60, 0x38, 0x0C, 0xC6, 0x7C, 0x00],
        'T' => [0x7E, 0x5A, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'U' => [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00],
        'V' => [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00],
        'W' => [0xC6, 0xC6, 0xC6, 0xD6, 0xFE, 0xEE, 0xC6, 0x00],
        'X' => [0xC6, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0xC6, 0x00],
        'Y' => [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x3C, 0x00],
        'Z' => [0xFE, 0xC6, 0x8C, 0x18, 0x32, 0x66, 0xFE, 0x00],
        '[' => [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00],
        '\\' => [0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x00, 0x00],
        ']' => [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00],
        '^' => [0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
        '`' => [0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00],
        'a' => [0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0x76, 0x00],
        'b' => [0xE0, 0x60, 0x7C, 0x66, 0x66, 0x66, 0xDC, 0x00],
        'c' => [0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC6, 0x7C, 0x00],
        'd' => [0x1C, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00],
        'e' => [0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0x7C, 0x00],
        'f' => [0x1C, 0x36, 0x30, 0x78, 0x30, 0x30, 0x78, 0x00],
        'g' => [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0x78],
        'h' => [0xE0, 0x60, 0x6C, 0x76, 0x66, 0x66, 0xE6, 0x00],
        'i' => [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'j' => [0x06, 0x00, 0x0E, 0x06, 0x06, 0x66, 0x66, 0x3C],
        'k' => [0xE0, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0xE6, 0x00],
        'l' => [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'm' => [0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xD6, 0x00],
        'n' => [0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x00],
        'o' => [0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0x00],
        'p' => [0x00, 0x00, 0xDC, 0x66, 0x66, 0x7C, 0x60, 0xF0],
        'q' => [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0x1E],
        'r' => [0x00, 0x00, 0xDC, 0x76, 0x60, 0x60, 0xF0, 0x00],
        's' => [0x00, 0x00, 0x7C, 0xC0, 0x7C, 0x06, 0xFC, 0x00],
        't' => [0x30, 0x30, 0x7C, 0x30, 0x30, 0x36, 0x1C, 0x00],
        'u' => [0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00],
        'v' => [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00],
        'w' => [0x00, 0x00, 0xC6, 0xD6, 0xD6, 0xFE, 0x6C, 0x00],
        'x' => [0x00, 0x00, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0x00],
        'y' => [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x7C],
        'z' => [0x00, 0x00, 0xFE, 0x8C, 0x18, 0x32, 0xFE, 0x00],
        '{' => [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00],
        '|' => [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00],
        '}' => [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00],
        '~' => [0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55], // Padrão checkerboard
    }
}

// =============================================================================
// API PÚBLICA
// =============================================================================

/// Inicializa o console de debug
/// COMENTE ESTA LINHA PARA DESATIVAR: // crate::core::debug::display::init();
pub fn init() {
    // Pegar informações do CRTC
    let crtc = crate::drivers::display::crtc::DISPLAY_CRTC.lock();
    let info = crtc.get_info();
    let fb_addr = crtc.framebuffer_ptr() as u64;
    drop(crtc);

    let mut console = DEBUG_CONSOLE.lock();
    console.fb_addr = fb_addr;
    console.width = info.width;
    console.height = info.height;
    console.stride = info.stride;
    console.max_cols = (info.width - 2 * MARGIN_X) / CHAR_WIDTH;
    console.max_rows = (info.height - 2 * MARGIN_Y) / CHAR_HEIGHT;
    console.cursor_x = 0;
    console.cursor_y = 0;
    console.initialized = true;
    drop(console);

    // Limpar tela
    clear();

    // Mostrar header
    log_color("[DEBUG CONSOLE ATIVO]", INFO_COLOR);
    log("=====================================");
}

/// Limpa a tela
pub fn clear() {
    let console = DEBUG_CONSOLE.lock();
    if !console.initialized {
        return;
    }

    let fb = console.fb_addr as *mut u32;
    let pixel_count = (console.width * console.height) as usize;

    unsafe {
        for i in 0..pixel_count {
            core::ptr::write_volatile(fb.add(i), BG_COLOR);
        }
    }
}

/// Rola a tela uma linha para cima (OTIMIZADO)
fn scroll_up() {
    let console = DEBUG_CONSOLE.lock();
    if !console.initialized {
        return;
    }

    let fb = console.fb_addr as *mut u8;
    let stride = console.stride as usize;
    let line_height = CHAR_HEIGHT as usize;
    let total_text_height = (console.max_rows as usize) * line_height;
    let width = console.width as usize;
    drop(console);

    unsafe {
        // Copiar tudo de uma vez (memcpy grande)
        let src_y = MARGIN_Y as usize + line_height;
        let dst_y = MARGIN_Y as usize;
        let copy_height = total_text_height - line_height;
        let total_bytes = copy_height * stride;

        let src = fb.add(src_y * stride);
        let dst = fb.add(dst_y * stride);

        // Uma única memcpy para todo o bloco
        core::ptr::copy(src, dst, total_bytes);

        // Limpar última linha de forma otimizada (uma linha inteira por vez)
        let console = DEBUG_CONSOLE.lock();
        let clear_y = MARGIN_Y as usize + total_text_height - line_height;
        let fb32 = console.fb_addr as *mut u32;
        let stride32 = stride / 4;

        // Limpar todas as linhas de pixels da última linha de texto
        for row in 0..line_height {
            let row_start = fb32.add((clear_y + row) * stride32);
            // Preencher linha inteira com cor de fundo
            for col in 0..width {
                *row_start.add(col) = BG_COLOR;
            }
        }
    }
}

/// Nova linha
/// Nova linha (SEM LOCK, deve ser chamada com lock já adquirido)
fn newline_unlocked(console: &mut DebugConsole) {
    console.cursor_x = 0;
    console.cursor_y += 1;

    if console.cursor_y >= console.max_rows {
        console.cursor_y = console.max_rows - 1;

        // Rolar tela (necessário liberar console momentaneamente para o scroll_up que trava internamente)
        // TODO: Otimizar scroll_up para não precisar de lock próprio
    }
}

/// Nova linha
fn newline() {
    let mut console = DEBUG_CONSOLE.lock();
    if !console.initialized {
        return;
    }

    console.cursor_x = 0;
    console.cursor_y += 1;

    if console.cursor_y >= console.max_rows {
        console.cursor_y = console.max_rows - 1;
        drop(console);
        scroll_up();
    }
}

/// Log de texto simples
pub fn log(msg: &str) {
    log_color(msg, TEXT_COLOR);
}

/// Log com cor customizada
pub fn log_color(msg: &str, color: u32) {
    let mut console = DEBUG_CONSOLE.lock();
    if !console.initialized || !console.visible {
        return;
    }

    let fb = console.fb_addr as *mut u32;
    let stride = console.stride;
    let max_cols = console.max_cols;

    for c in msg.chars() {
        if c == '\n' {
            console.cursor_x = 0;
            console.cursor_y += 1;
            if console.cursor_y >= console.max_rows {
                console.cursor_y = console.max_rows - 1;
                drop(console);
                scroll_up();
                console = DEBUG_CONSOLE.lock();
            }
            continue;
        }

        // Calcular posição do pixel
        let px = MARGIN_X + console.cursor_x * CHAR_WIDTH;
        let py = MARGIN_Y + console.cursor_y * CHAR_HEIGHT;

        // Desenhar caractere (libera lock pois draw_block_char é pura e lenta)
        let cx = console.cursor_x;
        let cy = console.cursor_y; // Keep cy for consistency with py calculation, though not directly used after drop(console)
        drop(console);
        draw_block_char(fb, stride, px, py, c, color);

        console = DEBUG_CONSOLE.lock();
        console.cursor_x = cx + 1;

        if console.cursor_x >= max_cols {
            console.cursor_x = 0;
            console.cursor_y += 1;
            if console.cursor_y >= console.max_rows {
                console.cursor_y = console.max_rows - 1;
                drop(console);
                scroll_up();
                console = DEBUG_CONSOLE.lock();
            }
        }
    }

    // Apenas pula linha se a mensagem não terminou em newline
    if !msg.ends_with('\n') {
        console.cursor_x = 0;
        console.cursor_y += 1;
        if console.cursor_y >= console.max_rows {
            console.cursor_y = console.max_rows - 1;
            drop(console);
            scroll_up();
        }
    }
}

/// Log de texto + valor hexadecimal
pub fn log_hex(msg: &str, value: u64) {
    // Formata a string e usa log_color para evitar código duplicado e bugs de lock
    // Como não temos format!, vamos fazer manual mas usando log_color interno sem quebra de linha
    log_color_no_newline(msg, TEXT_COLOR);
    log_color_no_newline(" 0x", INFO_COLOR);

    let mut hex_buf = [0u8; 16];
    let hex_chars = b"0123456789ABCDEF";
    for i in 0..16 {
        hex_buf[i] = hex_chars[((value >> ((15 - i) * 4)) & 0xF) as usize];
    }

    if let Ok(s) = core::str::from_utf8(&hex_buf) {
        log_color(s, INFO_COLOR); // Este último adiciona a quebra de linha
    }
}

/// Helper para log sem pular linha ao final
fn log_color_no_newline(msg: &str, color: u32) {
    let mut console = DEBUG_CONSOLE.lock();
    if !console.initialized || !console.visible {
        return;
    }

    let fb = console.fb_addr as *mut u32;
    let stride = console.stride;
    let max_cols = console.max_cols;

    for c in msg.chars() {
        if c == '\n' {
            console.cursor_x = 0;
            console.cursor_y += 1;
            if console.cursor_y >= console.max_rows {
                console.cursor_y = console.max_rows - 1;
                drop(console);
                scroll_up();
                console = DEBUG_CONSOLE.lock();
            }
            continue;
        }

        let px = MARGIN_X + console.cursor_x * CHAR_WIDTH;
        let py = MARGIN_Y + console.cursor_y * CHAR_HEIGHT;
        let cx = console.cursor_x;
        drop(console);
        draw_block_char(fb, stride, px, py, c, color);

        console = DEBUG_CONSOLE.lock();
        console.cursor_x = cx + 1;
        if console.cursor_x >= max_cols {
            console.cursor_x = 0;
            console.cursor_y += 1;
            if console.cursor_y >= console.max_rows {
                console.cursor_y = console.max_rows - 1;
                drop(console);
                scroll_up();
                console = DEBUG_CONSOLE.lock();
            }
        }
    }
}

/// Log de info (ciano)
pub fn info(msg: &str) {
    log_color(msg, INFO_COLOR);
}

/// Log de info com valor
pub fn info_hex(msg: &str, value: u64) {
    // Temporariamente mudar cor e chamar log_hex
    log_hex(msg, value);
}

/// Log de warning (amarelo)
pub fn warn(msg: &str) {
    log_color(msg, WARN_COLOR);
}

/// Log de erro (vermelho)
pub fn error(msg: &str) {
    log_color(msg, ERROR_COLOR);
}

/// Verifica se está inicializado
pub fn is_initialized() -> bool {
    DEBUG_CONSOLE.lock().initialized
}

/// Ativa a exibição de logs na tela
pub fn enable() {
    DEBUG_CONSOLE.lock().visible = true;
}

/// Desativa a exibição de logs na tela
pub fn disable() {
    DEBUG_CONSOLE.lock().visible = false;
}
