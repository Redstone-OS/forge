//! # Display Plane
//!
//! Um Plane é uma superfície de framebuffer renderizada pelo Display Engine.
//! Cada pipe pode ter múltiplos planes (Primary, Cursor, Sprites).
//!
//! ## Tipos de Plane
//!
//! - **Primary**: Framebuffer principal (desktop background)
//! - **Cursor**: Hardware cursor (64x64 ou 256x256)
//! - **Sprite**: Overlay planes para vídeo/composição

#![allow(dead_code)]

use super::super::hw::{regs, Mmio};

// =============================================================================
// PLANE TYPE
// =============================================================================

/// Tipo de plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneType {
    /// Plane principal (background).
    Primary,
    /// Hardware cursor.
    Cursor,
    /// Sprite overlay.
    Sprite(u8),
}

// =============================================================================
// PLANE
// =============================================================================

/// Display Plane (framebuffer surface).
#[derive(Clone)]
pub struct Plane {
    /// Pipe associado.
    pipe: u8,
    /// Tipo do plane.
    plane_type: PlaneType,
    /// MMIO.
    mmio: Mmio,
}

impl Plane {
    /// Cria novo plane.
    pub fn new(pipe: u8, plane_type: PlaneType, mmio: Mmio) -> Self {
        Self {
            pipe,
            plane_type,
            mmio,
        }
    }

    /// Habilita o plane.
    pub fn enable(&self) {
        let ctrl_reg = self.control_register();
        self.mmio.set_bits(ctrl_reg, regs::PLANE_ENABLE);
    }

    /// Desabilita o plane.
    pub fn disable(&self) {
        let ctrl_reg = self.control_register();
        self.mmio.clear_bits(ctrl_reg, regs::PLANE_ENABLE);
    }

    /// Verifica se está habilitado.
    pub fn is_enabled(&self) -> bool {
        let ctrl_reg = self.control_register();
        (self.mmio.read32(ctrl_reg) & regs::PLANE_ENABLE) != 0
    }

    /// Configura endereço da superfície (framebuffer).
    ///
    /// O endereço deve ser um offset na GTT, não endereço físico.
    pub fn set_surface(&self, gtt_offset: u64) {
        let surf_reg = self.surface_register();
        // Surface address é [31:12], bits [11:0] são reservados
        self.mmio
            .write32(surf_reg, (gtt_offset & 0xFFFF_F000) as u32);
    }

    /// Configura stride (bytes por linha).
    pub fn set_stride(&self, stride: u32) {
        let stride_reg = self.stride_register();
        // Stride é em múltiplos de 64 bytes
        self.mmio.write32(stride_reg, stride);
    }

    /// Configura formato de pixel.
    pub fn set_format(&self, format: u32) {
        let ctrl_reg = self.control_register();
        let current = self.mmio.read32(ctrl_reg);
        // Limpar bits de formato [29:26] e setar novo
        let new_value = (current & !(0xF << 26)) | format;
        self.mmio.write32(ctrl_reg, new_value);
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    /// Registrador de controle.
    fn control_register(&self) -> u32 {
        match self.plane_type {
            PlaneType::Primary => match self.pipe {
                0 => regs::DSPACNTR,
                1 => regs::DSPBCNTR,
                _ => regs::DSPACNTR,
            },
            PlaneType::Cursor => regs::CURACNTR + regs::pipe_offset(self.pipe),
            PlaneType::Sprite(_) => regs::DSPACNTR, // TODO: Sprite registers
        }
    }

    /// Registrador de surface address.
    fn surface_register(&self) -> u32 {
        match self.plane_type {
            PlaneType::Primary => match self.pipe {
                0 => regs::DSPASURF,
                1 => regs::DSPBSURF,
                _ => regs::DSPASURF,
            },
            PlaneType::Cursor => regs::CURABASE + regs::pipe_offset(self.pipe),
            PlaneType::Sprite(_) => regs::DSPASURF, // TODO
        }
    }

    /// Registrador de stride.
    fn stride_register(&self) -> u32 {
        match self.plane_type {
            PlaneType::Primary => regs::DSPASTRIDE + regs::pipe_offset(self.pipe),
            _ => regs::DSPASTRIDE, // Cursor não tem stride separado
        }
    }
}
