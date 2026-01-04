//! # Legacy VGA Register Definitions
//!
//! Contém definições de portas I/O e registros para compatibilidade VGA (0x3C0-0x3DF).

pub const VGA_AC_INDEX: u16 = 0x3C0;
pub const VGA_AC_WRITE: u16 = 0x3C0;
pub const VGA_AC_READ: u16 = 0x3C1;
pub const VGA_MISC_WRITE: u16 = 0x3C2;
pub const VGA_SEQ_INDEX: u16 = 0x3C4;
pub const VGA_SEQ_DATA: u16 = 0x3C5;
pub const VGA_DAC_READ_INDEX: u16 = 0x3C7;
pub const VGA_DAC_WRITE_INDEX: u16 = 0x3C8;
pub const VGA_DAC_DATA: u16 = 0x3C9;
pub const VGA_MISC_READ: u16 = 0x3CC;
pub const VGA_GC_INDEX: u16 = 0x3CE;
pub const VGA_GC_DATA: u16 = 0x3CF;

// Mapeamento CRTC (Color: 0x3D4, Mono: 0x3B4)
pub const VGA_CRTC_INDEX: u16 = 0x3D4;
pub const VGA_CRTC_DATA: u16 = 0x3D5;
pub const VGA_INSTAT_READ: u16 = 0x3DA;

pub struct VgaController;

impl VgaController {
    /// Desabilita o cursor de texto
    pub unsafe fn disable_cursor(&self) {
        use crate::arch::x86_64::ports::outb;
        outb(VGA_CRTC_INDEX, 0x0A);
        outb(VGA_CRTC_DATA, 0x20);
    }

    /// Limpa a memória de vídeo VGA (0xB8000)
    pub unsafe fn clear_text_mode(&self) {
        let ptr = 0xB8000 as *mut u16;
        for i in 0..(80 * 25) {
            ptr.add(i).write_volatile(0x0720); // Espaço em branco com cinza no preto
        }
    }
}
