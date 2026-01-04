//! # Generic Display Operations
//!
//! Implementação em software de primitivas gráficas (BitBlt, Alpha Blending).
//! Usado como fallback quando a GPU não possui aceleração de hardware ativa.

use gfx_types::{PixelFormat, Rect};

pub struct GenericOps;

impl GenericOps {
    /// Bit Block Transfer (Cópia simples de memória)
    pub unsafe fn bit_blt(
        src: *const u8,
        dst: *mut u8,
        src_stride: usize,
        dst_stride: usize,
        rect: Rect,
        bpp: usize,
    ) {
        for row in 0..rect.height as usize {
            let src_row = src.add((row + rect.y as usize) * src_stride + (rect.x as usize * bpp));
            let dst_row = dst.add((row + rect.y as usize) * dst_stride + (rect.x as usize * bpp));
            core::ptr::copy_nonoverlapping(src_row, dst_row, rect.width as usize * bpp);
        }
    }

    /// Preenche uma região com cor sólida
    pub unsafe fn fill_rect(dst: *mut u32, stride_pixels: usize, rect: Rect, color: u32) {
        for row in 0..rect.height as usize {
            let row_ptr = dst.add((row + rect.y as usize) * stride_pixels + rect.x as usize);
            for x in 0..rect.width as usize {
                row_ptr.add(x).write_volatile(color);
            }
        }
    }
}
