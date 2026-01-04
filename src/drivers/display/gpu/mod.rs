//! # Hardware GPU Drivers
//!
//! Drivers para GPUs físicas de alta performance.
//! Cada submódulo lida com as particularidades de cada fabricante.
//!
//! ## Fabricantes Suportados:
//! - **Intel**: HD Graphics, Iris, Arc
//! - **AMD**: Radeon
//! - **NVIDIA**: GeForce/Quadro

pub mod amd;
pub mod generic;
pub mod intel;
pub mod nvidia;

/// Inicializa todos os drivers de GPU.
pub fn init() {
    crate::kinfo!("(GPU) Registrando drivers de GPU...");

    // Ordem de prioridade: específicos primeiro, genérico por último
    intel::init();
    nvidia::init();
    amd::init();
    generic::init();
}
