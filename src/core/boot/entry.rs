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
    // DEBUG: Trap CPU here to verify entry point
    unsafe {
        core::arch::asm!("cli");
    }
    loop {
        core::hint::spin_loop();
    }

    // 1. Inicialização Precoce (Early Init) - Antes do Heap
    // Configurar Log Serial para que possamos ver o que está acontecendo.
    // (Serial driver geralmente não precisa de heap)
    crate::drivers::serial::init();
    crate::kinfo!("--- Iniciando Forge Kernel ---");

    // Validação da ABI do Bootloader
    if boot_info.magic != crate::core::boot::handoff::BOOT_INFO_MAGIC {
        crate::kerror!("PANIC: Invalid BootInfo Magic: {:#X}", boot_info.magic);
        loop {}
    }

    crate::kinfo!("Versão do Protocolo de Boot: {}", boot_info.version);

    // 2. Inicialização da Arquitetura (CPU, GDT, IDT, Interrupções)
    crate::kinfo!("Inicializando Arquitetura...");
    unsafe {
        crate::arch::init_basics(); // TODO: Expor init unificado em arch
    }

    // 2.5. Inicialização de Vídeo (Framebuffer)
    // Inicializamos cedo para ter saída visual se o serial falhar ou para mostrar logo
    crate::drivers::video::init(boot_info.framebuffer);

    // 3. Inicialização de Memória (PMM, VMM, Heap)
    crate::kinfo!("Inicializando Memória...");
    unsafe {
        // Agora passamos o boot_info completo. O mm::init deve saber lidar com
        // memory_map_addr e memory_map_len.
        crate::mm::init(boot_info);
    }

    // 4. Inicialização do Core (Time, SMP, Sched)
    crate::kinfo!("Inicializando Subsistemas do Núcleo...");
    crate::core::time::init();

    // 5. ACPI e Descoberta de Hardware
    crate::kinfo!("Inicializando ACPI...");
    if boot_info.rsdp_addr != 0 {
        // Inicializa ACPI via implementação da arquitetura (x86_64)
        crate::arch::platform::acpi::init(boot_info.rsdp_addr);
    }

    // 6. SMP Bringup (Acordar outros cores)
    crate::kinfo!("Inicializando SMP...");
    crate::core::smp::bringup::init();

    // 7. Executar Initcalls (Drivers, Filesystems, etc.)
    crate::kinfo!("Executando Initcalls...");
    crate::core::boot::initcall::run_initcalls();

    // 8. Inicialização do Userspace (Init Process)
    crate::kinfo!("Iniciando Processo Init...");
    crate::core::process::spawn_init();

    crate::kinfo!("Inicialização do Kernel Concluída. Entrando em Loop Ocioso.");

    // 9. Loop de Ociosidade (Idle Loop)
    // A thread de boot se torna a thread idle da CPU 0 (ou morre se spawnarmos uma task idle separada)
    cpuidle::enter_idle_loop();
}
