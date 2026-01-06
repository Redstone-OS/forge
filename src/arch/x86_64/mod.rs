//! Implementação x86_64

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod ports;
pub mod syscall;

pub mod acpi;
pub mod apic;
pub mod iommu;
pub mod smp;
pub mod vmm;

pub use cpu::Cpu;

/// Inicializa o básico da arquitetura: GDT, IDT, PICS, Syscall.
///
/// # Safety
///
/// Deve ser chamado no início do boot, single-core.
pub unsafe fn init_basics() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pics(); // Remapear PIC para 32-47

    // Inicializar PIT (Timer) - 250 Hz
    crate::drivers::system::timer::pit::init(250);

    // Inicializar syscall MSRs
    syscall::init();

    crate::kinfo!("(Arch) Instruções básicas inicializadas (GDT, IDT, Syscall)");
}
