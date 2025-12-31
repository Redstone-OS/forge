//! # CRTC - Display Output Controller
//!
//! Controla o output para o display físico.
//! Implementa double buffering via page flip.

use super::buffer::BUFFER_MANAGER;
use crate::core::boot::handoff::{
    FramebufferInfo as HandoffFbInfo, PixelFormat as HandoffPixelFormat,
};
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use gfx_types::{BufferHandle, DisplayInfo, PixelFormat, Rect};

// ============================================================================
// ERRORS
// ============================================================================

/// Erros do CRTC.
#[derive(Debug, Clone, Copy)]
pub enum CrtcError {
    /// Nenhum display inicializado.
    NoDisplay,
    /// Buffer inválido.
    InvalidBuffer,
    /// Page flip pendente.
    FlipPending,
    /// Falha na cópia.
    CopyFailed,
}

// ============================================================================
// CRTC
// ============================================================================

/// Controlador de display (CRTC).
pub struct Crtc {
    /// Informações do display.
    pub info: DisplayInfo,
    /// Endereço físico do framebuffer de hardware.
    fb_phys_addr: PhysAddr,
    /// Endereço virtual do framebuffer de hardware.
    fb_virt_addr: VirtAddr,
    /// Buffer atualmente sendo exibido.
    pub front_buffer: Option<BufferHandle>,
    /// Page flip pendente.
    pub pending_flip: bool,
    /// Contagem de frames.
    pub frame_count: u64,
}

impl Crtc {
    /// Cria novo CRTC não inicializado.
    const fn uninitialized() -> Self {
        Self {
            info: DisplayInfo {
                id: 0,
                width: 0,
                height: 0,
                refresh_rate_mhz: 60000,
                format: PixelFormat::ARGB8888,
                stride: 0,
            },
            fb_phys_addr: PhysAddr::new(0),
            fb_virt_addr: VirtAddr::new(0),
            front_buffer: None,
            pending_flip: false,
            frame_count: 0,
        }
    }

    /// Inicializa com informações do bootloader.
    pub fn init(&mut self, info: HandoffFbInfo) {
        let format = match info.format {
            HandoffPixelFormat::Rgb => PixelFormat::ARGB8888,
            HandoffPixelFormat::Bgr => PixelFormat::BGRA8888,
            _ => PixelFormat::ARGB8888,
        };

        let stride = info.stride * 4; // Converter de pixels para bytes

        self.info = DisplayInfo {
            id: 0,
            width: info.width,
            height: info.height,
            refresh_rate_mhz: 60000,
            format,
            stride,
        };

        // Mapear framebuffer físico para virtual
        let virt_addr = unsafe { crate::mm::addr::phys_to_virt::<u64>(info.addr) };

        self.fb_phys_addr = PhysAddr::new(info.addr);
        self.fb_virt_addr = VirtAddr::new(virt_addr as u64);

        crate::kinfo!("(CRTC) Inicializado:");
        crate::ktrace!("(CRTC) Resolução:", self.info.width as u64);
        crate::ktrace!("(CRTC) x", self.info.height as u64);
        crate::ktrace!("(CRTC) FB addr:", self.fb_virt_addr.as_u64());
    }

    /// Retorna informações do display.
    pub fn get_info(&self) -> DisplayInfo {
        self.info
    }

    /// Retorna ponteiro bruto para o framebuffer de hardware.
    /// Usado para compatibilidade com syscalls legados.
    pub fn framebuffer_ptr(&self) -> *mut u8 {
        self.fb_virt_addr.as_mut_ptr::<u8>()
    }

    /// Executa page flip - copia buffer para framebuffer de hardware.
    pub fn commit(&mut self, buffer_handle: BufferHandle) -> Result<(), CrtcError> {
        if self.pending_flip {
            return Err(CrtcError::FlipPending);
        }

        let buffer_mgr = BUFFER_MANAGER.lock();
        let buffer = buffer_mgr
            .get(buffer_handle)
            .ok_or(CrtcError::InvalidBuffer)?;

        // Copiar buffer para framebuffer de hardware
        self.copy_to_hardware(buffer)?;

        self.front_buffer = Some(buffer_handle);
        self.frame_count += 1;

        Ok(())
    }

    /// Executa page flip com damage regions (cópia parcial).
    pub fn commit_with_damage(
        &mut self,
        buffer_handle: BufferHandle,
        damage: &[Rect],
    ) -> Result<(), CrtcError> {
        if self.pending_flip {
            return Err(CrtcError::FlipPending);
        }

        let buffer_mgr = BUFFER_MANAGER.lock();
        let buffer = buffer_mgr
            .get(buffer_handle)
            .ok_or(CrtcError::InvalidBuffer)?;

        if damage.is_empty() {
            // Sem damage = copia tudo
            self.copy_to_hardware(buffer)?;
        } else {
            // Copia apenas as regiões danificadas
            for rect in damage {
                self.copy_region_to_hardware(buffer, rect)?;
            }
        }

        self.front_buffer = Some(buffer_handle);
        self.frame_count += 1;

        Ok(())
    }

    /// Copia buffer inteiro para hardware (otimizado).
    fn copy_to_hardware(&self, buffer: &super::buffer::DisplayBuffer) -> Result<(), CrtcError> {
        let size = buffer.desc.size_bytes();

        unsafe {
            // Usar copy_nonoverlapping para máxima performance
            core::ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                self.fb_virt_addr.as_mut_ptr::<u8>(),
                size,
            );
        }

        Ok(())
    }

    /// Copia região específica para hardware.
    fn copy_region_to_hardware(
        &self,
        buffer: &super::buffer::DisplayBuffer,
        rect: &Rect,
    ) -> Result<(), CrtcError> {
        let src_stride = buffer.desc.stride as usize;
        let dst_stride = self.info.stride as usize;
        let bpp = buffer.desc.format.bytes_per_pixel() as usize;

        // Clampar rect aos limites do buffer
        let x = rect.x.max(0) as usize;
        let y = rect.y.max(0) as usize;
        let width = (rect.width as usize).min(buffer.desc.width as usize - x);
        let height = (rect.height as usize).min(buffer.desc.height as usize - y);

        let bytes_per_row = width * bpp;

        unsafe {
            let src_base = buffer.as_ptr();
            let dst_base = self.fb_virt_addr.as_mut_ptr::<u8>();

            for row in 0..height {
                let src_offset = (y + row) * src_stride + x * bpp;
                let dst_offset = (y + row) * dst_stride + x * bpp;

                core::ptr::copy_nonoverlapping(
                    src_base.add(src_offset),
                    dst_base.add(dst_offset),
                    bytes_per_row,
                );
            }
        }

        Ok(())
    }

    /// Limpa o display com uma cor sólida.
    pub fn clear(&self, color: u32) {
        let pixel_count = (self.info.width * self.info.height) as usize;
        let ptr = self.fb_virt_addr.as_mut_ptr::<u32>();

        unsafe {
            for i in 0..pixel_count {
                core::ptr::write_volatile(ptr.add(i), color);
            }
        }
    }
}

// ============================================================================
// INICIALIZAÇÃO
// ============================================================================

/// CRTC global (um display por enquanto).
pub static DISPLAY_CRTC: Spinlock<Crtc> = Spinlock::new(Crtc::uninitialized());

/// Inicializa o CRTC.
pub fn init(info: HandoffFbInfo) {
    DISPLAY_CRTC.lock().init(info);
}
