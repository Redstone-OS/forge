//! # Display Pipe
//!
//! O Pipe é o gerador de timing do display.
//! Cada pipe processa uma stream de pixels e envia para um transcoder/encoder.
//!
//! ## Gen9 LP: 3 Pipes (A, B, C)

#![allow(dead_code)]

use super::super::hw::{regs, Mmio};

// =============================================================================
// PIPE
// =============================================================================

/// Display Pipe (timing generator).
#[derive(Clone)]
pub struct Pipe {
    /// Índice do pipe (0=A, 1=B, 2=C).
    index: u8,
    /// Referência ao MMIO.
    mmio: Mmio,
}

impl Pipe {
    /// Cria novo pipe.
    pub fn new(index: u8, mmio: Mmio) -> Self {
        Self { index, mmio }
    }

    /// Retorna índice do pipe.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Habilita o pipe.
    pub fn enable(&self) {
        let conf_reg = self.config_register();
        self.mmio.set_bits(conf_reg, regs::PIPE_ENABLE);

        // Esperar pipe ficar ativo
        self.mmio.wait_for_bits(conf_reg, regs::PIPE_STATE, 100);
    }

    /// Desabilita o pipe.
    pub fn disable(&self) {
        let conf_reg = self.config_register();
        self.mmio.clear_bits(conf_reg, regs::PIPE_ENABLE);

        // Esperar pipe ficar inativo
        for _ in 0..100 {
            if (self.mmio.read32(conf_reg) & regs::PIPE_STATE) == 0 {
                break;
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }
    }

    /// Verifica se pipe está habilitado.
    pub fn is_enabled(&self) -> bool {
        let conf_reg = self.config_register();
        (self.mmio.read32(conf_reg) & regs::PIPE_ENABLE) != 0
    }

    /// Configura resolução do pipe.
    pub fn set_resolution(&self, width: u32, height: u32) {
        let src_reg = self.source_register();
        // Formato: [31:16] = width-1, [15:0] = height-1
        let value = ((width - 1) << 16) | (height - 1);
        self.mmio.write32(src_reg, value);
    }

    /// Lê resolução atual.
    pub fn get_resolution(&self) -> (u32, u32) {
        let src_reg = self.source_register();
        let value = self.mmio.read32(src_reg);
        let width = ((value >> 16) & 0xFFF) + 1;
        let height = (value & 0xFFF) + 1;
        (width, height)
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    /// Retorna registrador de configuração para este pipe.
    fn config_register(&self) -> u32 {
        match self.index {
            0 => regs::PIPEACONF,
            1 => regs::PIPEBCONF,
            2 => regs::PIPECCONF,
            _ => regs::PIPEACONF,
        }
    }

    /// Retorna registrador de source size para este pipe.
    fn source_register(&self) -> u32 {
        match self.index {
            0 => regs::PIPEASRC,
            1 => regs::PIPEBSRC,
            2 => regs::PIPECSRC,
            _ => regs::PIPEASRC,
        }
    }
}
