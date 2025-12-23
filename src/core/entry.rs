// (FASE2) src/core/entry.rs
//! Entry Point Lógico do Kernel.
//!
//! Orquestra a inicialização de todos os subsistemas na ordem correta
//! de dependência.

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::core::handoff::{BootInfo, BOOT_MAGIC};

/// Função principal do Kernel (High-Level).
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Sanity Check (Segurança contra bootloaders incorretos)
    if boot_info.magic != BOOT_MAGIC {
        Cpu::hang();
    }

    // 2. Inicializar Logs (Serial)
    // Permite que vejamos o que está acontecendo daqui para frente.
    crate::kinfo!("==========================================");
    crate::kinfo!("Redstone OS Kernel (Forge) - Initializing");
    crate::kinfo!("Bootloader Protocol v{}", boot_info.version);

    // 3. Inicializar Arquitetura (GDT/IDT)
    // Essencial para proteção de memória e tratamento de exceções.
    crate::kinfo!("[Init] Arch: Setting up GDT/IDT...");
    unsafe {
        crate::arch::platform::gdt::init();
        crate::arch::platform::idt::init();
    }

    // 4. Memória (PMM, VMM, Heap)
    // Habilita alocação dinâmica (Vec, Box, Arc).
    crate::kinfo!("[Init] Memory: Initializing Subsystems...");
    crate::mm::init(boot_info);

    // 5. Drivers Básicos (PIC, PIT)
    // Prepara o sistema para receber interrupções de relógio.
    crate::kinfo!("[Init] Drivers: Configuring PIC and PIT...");
    unsafe {
        let mut pic = crate::drivers::pic::PICS.lock();
        pic.init();
        pic.unmask(0); // Habilita Timer (IRQ0)
                       // pic.unmask(1); // Futuro: Teclado (IRQ1)
    }

    {
        let mut pit = crate::drivers::timer::PIT.lock();
        let freq = pit.set_frequency(100); // 100Hz = 10ms quantum
        crate::kinfo!("[Init] Timer frequency set to {}Hz", freq);
    }

    // 6. IPC e Segurança
    // Inicializa estruturas lógicas.
    crate::ipc::init();

    // 7. Sistema de Arquivos (VFS & Initramfs)
    // Tenta montar o disco inicial na memória.
    crate::fs::init(boot_info);

    // 8. Scheduler (Multitarefa)
    // Cria a tarefa 'init' e prepara a fila de execução.
    crate::kinfo!("[Init] Scheduler: Spawning init tasks...");
    crate::sched::scheduler::init();

    // 9. Grande Salto (Enable Interrupts)
    // A partir daqui, o Timer vai disparar e o Scheduler vai assumir o controle.
    crate::kinfo!("[Init] Enabling Interrupts (STI) - System Live");
    Cpu::enable_interrupts();

    // Loop da thread "idle" (boot core)
    loop {
        Cpu::halt();
    }
}
