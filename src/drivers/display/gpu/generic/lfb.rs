//! # Linear Framebuffer (LFB) Management
//!
//! Lógica para gerenciar displays em modo gráfico usando mapeamento de memória direta.
//! Suporta as resoluções vindas do UEFI GOP ou VESA.

use crate::rmm::addr::{PhysAddr, VirtAddr};
// TODO: Revisar no futuro
#[allow(unused_imports)]
use gfx_types::{DisplayInfo, PixelFormat};

pub struct LfbController {
    pub phys_base: PhysAddr,
    pub virt_base: VirtAddr,
    pub size: usize,
    pub info: DisplayInfo,
}

impl LfbController {
    pub fn new(info: DisplayInfo, phys: PhysAddr) -> Self {
        // Mapear o framebuffer via HHDM ou mapeamento de dispositivo
        let virt = crate::rmm::virt::hhdm::phys_to_virt(phys.as_u64());

        Self {
            phys_base: phys,
            virt_base: VirtAddr::new(virt as u64),
            size: (info.stride * info.height) as usize,
            info,
        }
    }

    /// Executa um preenchimento rápido (Fast Clear)
    pub unsafe fn fill(&self, color: u32) {
        let ptr = self.virt_base.as_mut_ptr::<u32>();
        let count = (self.info.width * self.info.height) as usize;
        for i in 0..count {
            ptr.add(i).write_volatile(color);
        }
    }
}
