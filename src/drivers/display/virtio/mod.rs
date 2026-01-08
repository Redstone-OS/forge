//! # VirtIO-GPU Driver
//!
//! Driver para GPU virtualizada de alto desempenho (QEMU/KVM).
//! Implementa aceleração 2D básica e gerenciamento de framebuffer.
//!
//! ## Spec: OASIS VirtIO v1.2 - Section 5.7 GPU Device

#[path = "virtio-gpu/mod.rs"]
pub mod virtio_gpu;

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::display::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_GPU_DEVICE: u16 = 0x1050; // Legacy
const VIRTIO_GPU_DEVICE_MODERN: u16 = 0x1040 + 16; // Modern

/// Driver VirtIO-GPU para o RDS.
pub struct VirtioGpuDriver;

impl Driver for VirtioGpuDriver {
    fn name(&self) -> &'static str {
        "virtio-gpu"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let is_virtio_gpu = dev.vendor_id == VIRTIO_VENDOR
            && (dev.device_id == VIRTIO_GPU_DEVICE || dev.device_id == VIRTIO_GPU_DEVICE_MODERN);

        if !is_virtio_gpu {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(VirtIO-GPU) Detectado: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // TODO: Inicialização VirtIO:
        // 1. Reset device
        // 2. Negociar features
        // 3. Configurar virtqueues (controlq, cursorq)
        // 4. GetDisplayInfo
        // 5. Create resource 2D

        // Registrar dispositivo
        let device = VirtioGpuDevice::new();
        crate::drivers::display::register_device(Arc::new(device));

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::display::unregister_device("VirtIO-GPU");
        Ok(())
    }
}

struct VirtioGpuState {
    enabled: bool,
    width: u32,
    height: u32,
    stats: DisplayStats,
}

/// Dispositivo VirtIO-GPU.
pub struct VirtioGpuDevice {
    state: Spinlock<VirtioGpuState>,
}

impl VirtioGpuDevice {
    pub fn new() -> Self {
        Self {
            state: Spinlock::new(VirtioGpuState {
                enabled: false,
                width: 1024,
                height: 768,
                stats: DisplayStats::default(),
            }),
        }
    }
}

impl DisplayDevice for VirtioGpuDevice {
    fn name(&self) -> &str {
        "VirtIO-GPU"
    }

    fn info(&self) -> DisplayInfo {
        DisplayInfo {
            name: alloc::string::String::from("VirtIO-GPU"),
            model: alloc::string::String::from("VirtIO GPU (Paravirtualized)"),
            current_mode: self.current_mode(),
            framebuffer_addr: 0,
            stride: self.stride(),
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
                width: 1024,
                height: 768,
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
        // TODO: Send CMD_SET_SCANOUT
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
            accel_3d: false, // Virgl 3D é separado
            vram_size: 0,    // Host-backed
        }
    }

    fn framebuffer(&self) -> *mut u8 {
        // VirtIO-GPU não tem framebuffer direto, usa host memory
        core::ptr::null_mut()
    }

    fn framebuffer_size(&self) -> usize {
        let state = self.state.lock();
        (state.width * state.height * 4) as usize
    }

    fn stride(&self) -> u32 {
        self.state.lock().width * 4
    }

    fn clear(&self, _color: u32) {
        // TODO: Send 2D clear command
    }

    fn flip(&self) -> Result<(), DisplayError> {
        // TODO: Send CMD_RESOURCE_FLUSH
        self.state.lock().stats.flips += 1;
        Ok(())
    }

    fn enable(&self) -> Result<(), DisplayError> {
        self.state.lock().enabled = true;
        Ok(())
    }

    fn disable(&self) {
        self.state.lock().enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.state.lock().enabled
    }

    fn get_stats(&self) -> DisplayStats {
        self.state.lock().stats
    }
}

/// Registra o driver VirtIO-GPU.
pub fn init() {
    // crate::kinfo!("(VirtIO-GPU) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(VirtioGpuDriver));
}
