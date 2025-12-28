pub mod dsdt;
pub mod fadt;
/// Arquivo: x86_64/acpi/mod.rs
///
/// Propósito: Módulo de suporte a ACPI (Advanced Configuration and Power Interface).
/// O ACPI fornece tabelas de descrição de hardware essenciais para descobrir:
/// - Topologia de CPUs (MADT).
/// - Configuração de Energia (FADT).
/// - Dispositivos de Sistema (DSDT).
///
/// Módulos contidos:
/// - `madt`: Multiple APIC Description Table.
/// - `fadt`: Fixed ACPI Description Table.
/// - `dsdt`: Differentiated System Description Table.
pub mod madt;

/// Inicializa o subsistema ACPI
pub fn init(rsdp: u64) {
    crate::kinfo!("(ACPI) Init with RSDP: ", rsdp);
    // TODO: Parse RSDP, XSDT, FADT, MADT
}
