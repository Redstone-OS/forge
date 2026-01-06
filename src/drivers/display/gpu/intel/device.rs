//! # Intel GPU Device State
//!
//! Estrutura central do dispositivo Intel GPU.
//! Implementa o trait `DisplayDevice` para integração com o subsistema de display.

use super::display::Pipe;
use super::hw::regs;
use super::hw::Mmio;
use super::memory::Ggtt;
use crate::drivers::bus::pci::PciAddress;
use crate::drivers::display::traits::*;
use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// DEVICE STATE
// =============================================================================

/// Estado interno do dispositivo Intel.
struct DeviceState {
    /// Se o dispositivo está habilitado.
    enabled: bool,
    /// Resolução atual.
    width: u32,
    height: u32,
    /// Estatísticas.
    stats: DisplayStats,
}

/// Dispositivo Intel GPU.
pub struct IntelDevice {
    /// Device ID PCI.
    device_id: u16,
    /// MMIO registers.
    mmio: Mmio,
    /// Endereço físico do aperture (framebuffer).
    aperture_phys: u64,
    /// Endereço PCI.
    #[allow(dead_code)]
    pci_addr: PciAddress,
    /// GTT global.
    #[allow(dead_code)]
    ggtt: Option<Ggtt>,
    /// Pipe A.
    #[allow(dead_code)]
    pipe_a: Pipe,
    /// Estado protegido.
    state: Spinlock<DeviceState>,
}

impl IntelDevice {
    /// Cria e inicializa um novo dispositivo Intel GPU.
    pub fn new(
        device_id: u16,
        mmio: Mmio,
        aperture_phys: u64,
        pci_addr: PciAddress,
    ) -> Option<Self> {
        crate::kdebug!("(Intel GPU) Inicializando dispositivo...");

        // Criar pipe principal
        let pipe_a = Pipe::new(0, mmio.clone());

        // Ler resolução atual do hardware (se já configurado pelo firmware)
        let (width, height) = Self::read_current_mode(&mmio);

        crate::kdebug!("(Intel GPU) Resolução atual:", width as u64);
        crate::kdebug!("(Intel GPU) x", height as u64);

        Some(Self {
            device_id,
            mmio,
            aperture_phys,
            pci_addr,
            ggtt: None, // Inicializado sob demanda
            pipe_a,
            state: Spinlock::new(DeviceState {
                enabled: true,
                width,
                height,
                stats: DisplayStats::default(),
            }),
        })
    }

    /// Lê modo atual do hardware.
    fn read_current_mode(mmio: &Mmio) -> (u32, u32) {
        // Ler PIPEASRC para obter resolução
        let src = mmio.read32(regs::PIPEASRC);
        if src == 0 {
            // Fallback se não configurado
            return (1024, 768);
        }

        let width = ((src >> 16) & 0xFFF) + 1;
        let height = (src & 0xFFF) + 1;

        (width, height)
    }

    /// Retorna endereço físico do aperture.
    pub fn aperture_address(&self) -> u64 {
        self.aperture_phys
    }
}

// =============================================================================
// DISPLAY DEVICE TRAIT
// =============================================================================

impl DisplayDevice for IntelDevice {
    fn name(&self) -> &str {
        "Intel HD Graphics"
    }

    fn info(&self) -> DisplayInfo {
        let state = self.state.lock();
        DisplayInfo {
            name: String::from("Intel HD Graphics"),
            model: String::from(super::hw::gen9::device_name(self.device_id)),
            current_mode: VideoMode {
                width: state.width,
                height: state.height,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000,
            },
            framebuffer_addr: self.aperture_phys,
            stride: state.width * 4,
        }
    }

    fn current_mode(&self) -> VideoMode {
        let state = self.state.lock();
        VideoMode {
            width: state.width,
            height: state.height,
            format: PixelFormat::Argb8888,
            refresh_rate_mhz: 60000,
        }
    }

    fn supported_modes(&self) -> Vec<VideoMode> {
        alloc::vec![
            VideoMode {
                width: 800,
                height: 600,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
            VideoMode {
                width: 1024,
                height: 768,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
            VideoMode {
                width: 1280,
                height: 720,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
            VideoMode {
                width: 1280,
                height: 800,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
            VideoMode {
                width: 1920,
                height: 1080,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
        ]
    }

    fn set_mode(&self, mode: VideoMode) -> Result<(), DisplayError> {
        // TODO: Implementar mode setting via Display Engine
        let mut state = self.state.lock();
        state.width = mode.width;
        state.height = mode.height;
        Ok(())
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities {
            modes: self.supported_modes(),
            max_width: 4096,
            max_height: 4096,
            double_buffer: true,
            page_flip: true,
            hw_cursor: true,
            accel_2d: true,
            accel_3d: false,              // Render engine não implementado ainda
            vram_size: 256 * 1024 * 1024, // Placeholder
        }
    }

    fn framebuffer(&self) -> *mut u8 {
        if self.aperture_phys == 0 {
            return core::ptr::null_mut();
        }
        // Mapear aperture para espaço virtual
        let virt = crate::rmm::virt::hhdm::phys_to_virt(self.aperture_phys);
        virt as *mut u8
    }

    fn framebuffer_size(&self) -> usize {
        let state = self.state.lock();
        (state.width * state.height * 4) as usize
    }

    fn stride(&self) -> u32 {
        self.state.lock().width * 4
    }

    fn clear(&self, color: u32) {
        let fb = self.framebuffer();
        if fb.is_null() {
            return;
        }

        let state = self.state.lock();
        let pixels = (state.width * state.height) as usize;

        unsafe {
            let ptr = fb as *mut u32;
            for i in 0..pixels {
                core::ptr::write_volatile(ptr.add(i), color);
            }
        }
    }

    fn enable(&self) -> Result<(), DisplayError> {
        // Habilitar pipe A
        self.mmio.set_bits(regs::PIPEACONF, regs::PIPE_ENABLE);
        self.state.lock().enabled = true;
        Ok(())
    }

    fn disable(&self) {
        // Desabilitar pipe A
        self.mmio.clear_bits(regs::PIPEACONF, regs::PIPE_ENABLE);
        self.state.lock().enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.state.lock().enabled
    }

    fn get_stats(&self) -> DisplayStats {
        self.state.lock().stats
    }
}
