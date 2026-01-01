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
use crate::core::power::cpuidle;

/// Ponto de entrada do Kernel Rust.
/// O Bootloader configura a stack e salta para cá.
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Inicialização Precoce (Early Init) - Antes do Heap
    // Configurar Log Serial para que possamos ver o que está acontecendo.
    // (Serial driver geralmente não precisa de heap)
    crate::drivers::serial::init();
    crate::kinfo!("--- 'Iniciando Forge Kernel' ---");

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
        crate::mm::init(boot_info);
    }

    // 2.5. Inicialização de Vídeo (Framebuffer)
    // Inicializamos agora que o HHDM está pronto para mapear o FB corretamente
    crate::drivers::display::init(boot_info.framebuffer);

    // 4. Inicialização do Core (Time, SMP, Sched)
    crate::kinfo!("'Inicializando Subsistemas do Núcleo'");
    crate::core::time::init();

    // 5. ACPI e Descoberta de Hardware
    crate::kinfo!("'Inicializando ACPI'");
    if boot_info.rsdp_addr != 0 {
        // Inicializa ACPI via implementação da arquitetura (x86_64)
        crate::arch::platform::acpi::init(boot_info.rsdp_addr);
    }

    // 6. SMP Bringup (Acordar outros cores)
    crate::kinfo!("'Inicializando SMP'");
    crate::core::smp::bringup::init();

    // 6.5 Inicializar VFS (Sistema de Arquivos Virtual)
    // Necessário antes de qualquer operação de arquivo
    crate::fs::vfs::init();

    // 7. Executar Initcalls (Drivers, Filesystems, etc.)

    crate::kinfo!("'Executando Initcalls'");
    crate::core::boot::initcall::run_initcalls();

    // 7.5. Inicializar InitRAMFS
    if boot_info.initramfs_addr != 0 && boot_info.initramfs_size > 0 {
        crate::kinfo!("'Inicializando InitRAMFS'");
        // SEMPRE acessar via HHDM para evitar depender do identity map legado
        let phys = boot_info.initramfs_addr;
        let virt = unsafe { crate::mm::addr::phys_to_virt::<u8>(phys) };
        let addr = crate::mm::VirtAddr::new(virt as u64);
        crate::fs::initramfs::init(addr, boot_info.initramfs_size as usize);
    } else {
        crate::kwarn!("InitRAMFS não encontrado!");
    }

    // 8. Inicialização do Userspace (Init Process)
    // Primeiro inicializar drivers de input
    crate::kinfo!("'Inicializando Drivers de Input'");
    crate::drivers::input::init();

    crate::kinfo!("'Iniciando Processo Init'");
    crate::core::process::spawn_init();

    crate::kinfo!("'Inicialização do Kernel Concluída'");

    // 9. Habilitar Timer IRQ (APÓS scheduler estar pronto)
    // O timer dispara e chama schedule(), então só habilitamos após spawn_init
    crate::kinfo!("'Habilitando Timer Preemptivo'");
    crate::arch::x86_64::interrupts::pic_enable_irq(0);

    // 10. Loop de Ociosidade (Idle Loop)
    cpuidle::enter_idle_loop();
}
