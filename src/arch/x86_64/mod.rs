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

pub use cpu::Cpu;

/// Inicializa o básico da arquitetura: GDT, IDT, PICS, Syscall.
///
/// # Safety
///
/// Deve ser chamado no início do boot, single-core.
pub unsafe fn init_basics() {
    gdt::init();
    interrupts::init_idt();
    // TODO: interrupts::init_pics(); // Se houver PIC legado.
    // PICS geralmente são inicializados e *desabilitados* se usarmos APIC.
    // Mas se o sistema depende de PIC inicialmente, deve inicializar.
    // init_pics não existe em interrupts.rs ainda, então comentamos.

    // Inicializar syscall MSRs
    syscall::init();

    crate::kinfo!("(Arch) Basics initialized (GDT, IDT, Syscall)");
}
