//! # Bochs Graphics Adapter (BGA) Driver
//!
//! Driver para o controlador de vídeo virtual Bochs/QEMU.
//! Suporta modos de 32 bits com Linear Framebuffer (LFB).
//!
//! ## PCI ID: 1234:1111
//! ## Detecção: VBE Dispi registers (ports 0x1CE/0x1CF)

use crate::arch::x86_64::ports::{inw, outw};
use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::display::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// VBE Dispi I/O ports
const INDEX_PORT: u16 = 0x01CE;
const DATA_PORT: u16 = 0x01CF;

// VBE Dispi register indices
const VBE_DISPI_INDEX_ID: u16 = 0;
const VBE_DISPI_INDEX_XRES: u16 = 1;
const VBE_DISPI_INDEX_YRES: u16 = 2;
const VBE_DISPI_INDEX_BPP: u16 = 3;
const VBE_DISPI_INDEX_ENABLE: u16 = 4;
const VBE_DISPI_INDEX_BANK: u16 = 5;
const VBE_DISPI_INDEX_VIRT_WIDTH: u16 = 6;
const VBE_DISPI_INDEX_VIRT_HEIGHT: u16 = 7;
const VBE_DISPI_INDEX_X_OFFSET: u16 = 8;
const VBE_DISPI_INDEX_Y_OFFSET: u16 = 9;

// VBE Dispi IDs
const VBE_DISPI_ID0: u16 = 0xB0C0;
const VBE_DISPI_ID5: u16 = 0xB0C5;

// Enable flags
const VBE_DISPI_DISABLED: u16 = 0x00;
const VBE_DISPI_ENABLED: u16 = 0x01;
const VBE_DISPI_LFB_ENABLED: u16 = 0x40;

/// Driver Bochs BGA para o RDS.
pub struct BochsDriver;

impl BochsDriver {
    fn write_reg(index: u16, data: u16) {
        unsafe {
            outw(INDEX_PORT, index);
            outw(DATA_PORT, data);
        }
    }

    fn read_reg(index: u16) -> u16 {
        unsafe {
            outw(INDEX_PORT, index);
            inw(DATA_PORT)
        }
    }

    fn check_version() -> bool {
        let id = Self::read_reg(VBE_DISPI_INDEX_ID);
        id >= VBE_DISPI_ID0 && id <= VBE_DISPI_ID5
    }
}

impl Driver for BochsDriver {
    fn name(&self) -> &'static str {
        "bochs-bga"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // Verificar PCI ID (1234:1111)
        if dev.vendor_id != 0x1234 || dev.device_id != 0x1111 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(Bochs BGA) Detectado no PCI");

        // Verificar hardware via I/O
        if !Self::check_version() {
            return Err(DriverError::HardwareFault);
        }

        // Criar dispositivo
        let device = BochsDevice::new();

        // Configurar modo inicial
        device.set_mode_internal(1024, 768, 32);

        // Registrar no subsistema
        crate::drivers::display::register_device(Arc::new(device));

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::display::unregister_device("Bochs BGA");
        Ok(())
    }
}

struct BochsState {
    enabled: bool,
    width: u32,
    height: u32,
    bpp: u8,
    fb_addr: u64,
    stats: DisplayStats,
}

/// Dispositivo Bochs BGA.
pub struct BochsDevice {
    state: Spinlock<BochsState>,
}

impl BochsDevice {
    pub fn new() -> Self {
        Self {
            state: Spinlock::new(BochsState {
                enabled: false,
                width: 1024,
                height: 768,
                bpp: 32,
                fb_addr: 0,
                stats: DisplayStats::default(),
            }),
        }
    }

    fn set_mode_internal(&self, width: u16, height: u16, bpp: u16) {
        BochsDriver::write_reg(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
        BochsDriver::write_reg(VBE_DISPI_INDEX_XRES, width);
        BochsDriver::write_reg(VBE_DISPI_INDEX_YRES, height);
        BochsDriver::write_reg(VBE_DISPI_INDEX_BPP, bpp);
        BochsDriver::write_reg(
            VBE_DISPI_INDEX_ENABLE,
            VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED,
        );

        let mut state = self.state.lock();
        state.width = width as u32;
        state.height = height as u32;
        state.bpp = bpp as u8;
        state.enabled = true;
    }
}

impl DisplayDevice for BochsDevice {
    fn name(&self) -> &str {
        "Bochs BGA"
    }

    fn info(&self) -> DisplayInfo {
        let state = self.state.lock();
        DisplayInfo {
            name: alloc::string::String::from("Bochs BGA"),
            model: alloc::string::String::from("VBE Dispi Virtual Display"),
            current_mode: self.current_mode(),
            framebuffer_addr: state.fb_addr,
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
                width: 640,
                height: 480,
                format: PixelFormat::Argb8888,
                refresh_rate_mhz: 60000
            },
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
                height: 1024,
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
        self.set_mode_internal(
            mode.width as u16,
            mode.height as u16,
            mode.format.bits_per_pixel() as u16,
        );
        Ok(())
    }

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities {
            modes: self.supported_modes(),
            max_width: 2560,
            max_height: 1600,
            double_buffer: false,
            page_flip: false,
            hw_cursor: false,
            accel_2d: false,
            accel_3d: false,
            vram_size: 16 * 1024 * 1024,
        }
    }

    fn framebuffer(&self) -> *mut u8 {
        // TODO: Mapear do PCI BAR0
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
        // TODO: Implementar clear via framebuffer
    }

    fn enable(&self) -> Result<(), DisplayError> {
        BochsDriver::write_reg(
            VBE_DISPI_INDEX_ENABLE,
            VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED,
        );
        self.state.lock().enabled = true;
        Ok(())
    }

    fn disable(&self) {
        BochsDriver::write_reg(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
        self.state.lock().enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.state.lock().enabled
    }

    fn get_stats(&self) -> DisplayStats {
        self.state.lock().stats
    }
}

/// Registra o driver Bochs.
pub fn init() {
    crate::kinfo!("(Bochs BGA) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(BochsDriver));
}
