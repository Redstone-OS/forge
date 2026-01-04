//! # Hardware-specific GPU Drivers
//!
//! Contém drivers para GPUs físicas de alta performance.
//! Cada submódulo lida com as particularidades de arquitetura de cada fabricante.

pub mod amd;
pub mod generic;
pub mod intel;
pub mod nvidia;

/// Inicializa todos os drivers de GPU conhecidos.
pub fn init() {
    intel::init();
    nvidia::init();
    amd::init();
    generic::init();
}
