//! Framebuffer linear

use crate::core::boot::handoff::{FramebufferInfo as HandoffFbInfo, PixelFormat};
use crate::mm::VirtAddr;
use crate::sync::Spinlock;

/// Informações do framebuffer (Internal Driver Struct)
#[derive(Clone, Copy)]
pub struct FramebufferInfo {
    pub addr: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32, // bytes por linha
    pub bpp: u32,    // bits por pixel
    pub format: PixelFormat,
}

static FRAMEBUFFER: Spinlock<Option<FramebufferInfo>> = Spinlock::new(None);

/// Inicializa com informações do bootloader
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(FB) Inicializando Framebuffer...");
    crate::ktrace!("(FB) Width:", info.width as u64);
    crate::ktrace!("(FB) Height:", info.height as u64);

    let bpp = match info.format {
        PixelFormat::Rgb | PixelFormat::Bgr | PixelFormat::Bitmask => 32,
        PixelFormat::BltOnly => 0, // No direct access
    };

    // Mapeamento Identidade para boot (assumindo que já está mapeado ou usando PhysAddr como VirtAddr temporariamente)
    // TODO: Usar VMM para mapear se necessário.
    // O bootloader IGNITE mapeia tudo identidade nas regiões inferiores?
    // ignite mapeia o framebuffer? Sim, geralmente.
    // SAFETY: phys_to_virt is unsafe, but here we assume identity mapping from bootloader
    let virt_addr = unsafe { crate::mm::addr::phys_to_virt::<u64>(info.addr) };

    let fb_info = FramebufferInfo {
        addr: VirtAddr::new(virt_addr as u64),
        width: info.width,
        height: info.height,
        stride: info.stride,
        bpp,
        format: info.format,
    };

    *FRAMEBUFFER.lock() = Some(fb_info);
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
        unsafe {
            *ptr = color;
        }
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
