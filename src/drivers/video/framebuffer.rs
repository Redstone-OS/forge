//! Framebuffer linear

use crate::mm::VirtAddr;
use crate::sync::Spinlock;

/// Informações do framebuffer
pub struct FramebufferInfo {
    pub addr: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,  // bytes por linha
    pub bpp: u32,     // bits por pixel
}

static FRAMEBUFFER: Spinlock<Option<FramebufferInfo>> = Spinlock::new(None);

/// Inicializa com informações do bootloader
pub fn init(info: FramebufferInfo) {
    *FRAMEBUFFER.lock() = Some(info);
    crate::kinfo!("(FB) Inicializado:", info.width as u64);
}

/// Escreve pixel
pub fn put_pixel(x: u32, y: u32, color: u32) {
    let fb = FRAMEBUFFER.lock();
    if let Some(ref info) = *fb {
        if x >= info.width || y >= info.height {
            return;
        }
        
        // Cuidado com overflow se u32
        let offset = (y as u64 * info.stride as u64 + x as u64 * (info.bpp as u64 / 8)) as usize;
        let ptr = info.addr.offset(offset as u64).as_mut_ptr::<u32>();
        
        // SAFETY: offset foi validado (aproximadamente, assumindo mapeamento correto)
        unsafe { *ptr = color; }
    }
}

/// Preenche retângulo
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            put_pixel(x + dx, y + dy, color);
        }
    }
}

/// Limpa tela
pub fn clear(color: u32) {
    let fb = FRAMEBUFFER.lock();
    if let Some(ref info) = *fb {
        fill_rect(0, 0, info.width, info.height, color);
    }
}
