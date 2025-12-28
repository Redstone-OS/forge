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
pub mod fadt;
pub mod dsdt;
