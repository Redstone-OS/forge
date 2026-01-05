//! # NUMA Topology
//!
//! Detecção de topologia NUMA via ACPI SRAT.

/// Detecta topologia NUMA via ACPI
pub fn detect_topology() {
    // TODO: Parsear ACPI SRAT
    crate::kinfo!("(RMM/NUMA) Topologia: 1 node (NUMA disabled)");
}
