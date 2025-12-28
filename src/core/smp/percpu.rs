//! Per-CPU Data - Dados locais por CPU
//!
//! TODO: Implementar
//! - Struct PerCpu { cpu_id, current_task, idle_task, local_apic_id }
//! - Macro percpu!() ou função current_cpu()
//! - Array estático [PerCpu; MAX_CPUS]
//! - Inicialização durante bringup de cada CPU
