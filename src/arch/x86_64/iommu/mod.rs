/// Arquivo: x86_64/iommu/mod.rs
///
/// Propósito: Módulo de gerenciamento de IOMMU (Input-Output Memory Management Unit).
/// A IOMMU é responsável por traduzir endereços de memória para dispositivos DMA (Direct Memory Access)
/// e prover isolamento/proteção (DMA Remapping).
///
/// Módulos contidos:
/// - `intel_vtd`: Implementação específica para Intel VT-d.

pub mod intel_vtd;
