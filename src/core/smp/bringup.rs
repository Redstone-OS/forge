//! CPU Bringup - Inicialização de CPUs secundárias (AP)
//!
//! TODO: Implementar
//! - Detectar número de CPUs via ACPI MADT
//! - Alocar stacks para cada AP
//! - Enviar INIT IPI + SIPI para cada AP
//! - Trampoline code em low memory (<1MB)
//! - Barreira de sincronização entre CPUs
