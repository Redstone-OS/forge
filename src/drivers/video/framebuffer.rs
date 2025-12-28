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
    let mut fb_guard = FRAMEBUFFER.lock();
    if let Some(ref mut info) = *fb_guard {
        if x >= info.width || y >= info.height {
            return;
        }
        
        let offset = (y * info.stride + x * (info.bpp / 8)) as usize;
        let ptr = info.addr.offset(offset as u64).as_mut_ptr::<u32>();
        
        // SAFETY: offset foi validado
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
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref info) = *fb_guard {
        let w = info.width;
        let h = info.height;
        drop(fb_guard); // Release lock before calling fill_rect to avoid deadlock if recursive (though fill_rect calls put_pixel which locks again... wait. put_pixel locks. fill_rect needs to NOT lock if it calls put_pixel? No, put_pixel implementation shown locks. So fill_rect calling put_pixel is fine if lock is Reentrant or if we drop lock. But here put_pixel locks internally. So fill_rect calling put_pixel 1000 times will lock/unlock 1000 times. That's fine for now, albeit slow. BUT clear() keeps the lock? No, I dropped it. BUT wait, the code given in guide:
        /*
        pub fn clear(color: u32) {
            let fb = FRAMEBUFFER.lock();
            if let Some(ref info) = *fb {
                fill_rect(0, 0, info.width, info.height, color);
            }
        }
        */
        // If fill_rect calls put_pixel, and put_pixel locks, we have DEADLOCK if clear() holds the lock.
        // Guide code for clear() holds 'fb'. fill_rect calls put_pixel. put_pixel locks FRAMEBUFFER. DEADLOCK.
        // I must fix this logic to match safety even if guide implies deadlock, or assume Spinlock is reentrant? Typically Spinlock is NOT reentrant.
        // However, I must follow guide "LITERALMENTE".
        // BUT rule says "NUNCA use polling infinito/timeout". Deadlock is infinite polling.
        // I will implement helper to avoid deadlock or copy struct data.
        // Guide code is problematic. I'll copy the width/height then drop lock, then call fill_rect.
        // Actually, looking at guide `put_pixel`:
        /*
        pub fn put_pixel(x: u32, y: u32, color: u32) {
            let fb = FRAMEBUFFER.lock();
            ...
        }
        */
        // I will stick to guide but I have to make it work. I will modify `fill_rect` to NOT call `put_pixel` but do it efficiently or unsafe internal?
        // No, I'll make `clear` drop the lock.
        
        fill_rect(0, 0, w, h, color);
    }
}
