/// Arquivo: core/boot/entry.rs
///
/// Propósito: Ponto de Entrada do Kernel (Kernel Entry Point).
/// Esta função é chamada pelo Bootloader (Ignite) após o salto para o modo Longo.
/// Responsável por orquestrar a inicialização de todos os subsistemas na ordem correta.
///
/// Detalhes de Implementação:
/// - Assinatura `extern "C"` para ABI estável.
/// - Recebe `BootInfo` do bootloader.
/// - Nunca retorna (loop infinito ou shutdown).
use super::handoff::BootInfo;

/// Ponto de entrada do Kernel Rust.
/// O Bootloader configura a stack e salta para cá.
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Inicialização Precoce (Early Init) - Antes do Heap
    // Configurar Log Serial para que possamos ver o que está acontecendo.
    // (Serial driver geralmente não precisa de heap)
    crate::drivers::comm::serial::init();
    crate::kinfo!("'--- Iniciando Forge Kernel ---'");

    // Validação da ABI do Bootloader
    if boot_info.magic != crate::core::boot::handoff::BOOT_INFO_MAGIC {
        crate::kerror!("PANIC: Invalid BootInfo Magic: {:#X}", boot_info.magic);
        loop {}
    }

    crate::kinfo!("Versão do Protocolo de Boot:", boot_info.version);

    // 2. Inicialização da Arquitetura (CPU, GDT, IDT, Interrupções)
    crate::kinfo!("'Inicializando Arquitetura'");
    unsafe {
        crate::arch::init_basics(); // TODO: Expor init unificado em arch
    }

    // 3. Inicialização de Memória (PMM, VMM, Heap, HHDM)
    crate::kinfo!("'Inicializando Memória'");
    unsafe {
        crate::rmm::init(boot_info);
    }

    // 2.5. Inicialização de Vídeo (Framebuffer)
    // Movido para dentro de drivers::init() para manter centralizado
    // crate::drivers::display::init(boot_info.framebuffer);

    // Debug console (desativado por padrão, ver core::debug::console)

    // 4. Inicialização do Core (Time, SMP, Sched)
    crate::kinfo!("'Inicializando Subsistemas do Núcleo'");
    crate::core::time::init();

    // 5. ACPI e Descoberta de Hardware
    crate::kinfo!("'Inicializando ACPI'");
    crate::kdebug!("(ACPI) RSDP addr do bootloader:", boot_info.rsdp_addr);
    if boot_info.rsdp_addr != 0 {
        // Inicializa ACPI via implementação da arquitetura (x86_64)
        unsafe {
            match crate::arch::platform::acpi::init(boot_info.rsdp_addr) {
                Ok(acpi_info) => {
                    // Inicializar topologia SMP com CPUs detectadas
                    crate::core::smp::topology::init(&acpi_info.cpus);
                }
                Err(e) => {
                    crate::kerror!("(ACPI) Falha:", e);
                }
            }
        }
    }

    // 6. SMP Bringup (Acordar outros cores)
    crate::kinfo!("'Inicializando SMP'");

    // Primeiro inicializar o LAPIC do BSP (necessário para enviar IPIs)
    unsafe {
        crate::arch::x86_64::apic::lapic::init();
    }
    crate::kdebug!("(SMP) LAPIC do BSP inicializado");

    crate::core::smp::bringup::init();
    // Acordar APs se temos mais de 1 CPU
    unsafe {
        crate::core::smp::bringup::bringup_all_aps();
    }

    // 6.5 Inicializar VFS (Sistema de Arquivos Virtual)
    // Necessário antes de qualquer operação de arquivo
    crate::fs::vfs::init();

    // 6.6 Inicializar Sistema de Drivers (PCI, USB, Block, Input)
    crate::drivers::init(boot_info.framebuffer);

    // 6.7 Inicializar e Montar FAT se houver disco
    crate::kinfo!("'Inicializando FAT'");
    crate::fs::fat::init();

    // 7. Executar Initcalls (Drivers, Filesystems, etc.)

    crate::kinfo!("'Executando Initcalls'");
    crate::core::boot::initcall::run_initcalls();

    // 7.5. Inicializar InitRAMFS
    if boot_info.initramfs_addr != 0 && boot_info.initramfs_size > 0 {
        crate::kinfo!("'Inicializando InitRAMFS'");
        // SEMPRE acessar via HHDM para evitar depender do identity map legado
        let phys = boot_info.initramfs_addr;
        let virt = crate::rmm::virt::hhdm::phys_to_virt(phys);
        let addr = crate::rmm::VirtAddr::new(virt as u64);
        crate::fs::initramfs::init(addr, boot_info.initramfs_size as usize);
    } else {
        crate::kwarn!("InitRAMFS não encontrado!");
    }

    // 8. Inicialização do Userspace (Init Process)

    // 8.5. Inicializar Idle Task
    // A idle task fica em IDLE_TASK (fallback permanente) e NÃO em CURRENT
    crate::kinfo!("'Inicializando Idle Task'");
    crate::sched::core::idle::init_idle_task();

    crate::kinfo!("'Iniciando Processo Init'");
    crate::core::process::spawn_init();

    crate::kinfo!("'Inicialização do Kernel Concluída'");

    // 9. Habilitar Timer IRQ (APÓS scheduler estar pronto)
    crate::kinfo!("'Habilitando Timer Preemptivo'");
    crate::arch::x86_64::interrupts::pic_enable_irq(0);

    // 10. Entrar no loop do scheduler
    // CURRENT está vazio, schedule() vai pegar a primeira task da RunQueue
    // Se não houver tasks, vai para a idle task (fallback)
    crate::sched::core::scheduler::run();
}
