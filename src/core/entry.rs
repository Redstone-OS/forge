//! Entry Point Lógico do Kernel.
//!
//! Este módulo contém a função `kernel_main`, que é o primeiro código Rust de alto nível
//! a ser executado após o "trampolim" em assembly (`_start`).
//!
//! # Responsabilidades
//! 1. **Validação**: Verifica se o Bootloader passou informações coerentes.
//! 2. **Orquestração**: Inicializa subsistemas na ordem estrita de dependência (Arch -> Memória -> Drivers -> Sched).
//! 3. **Transição**: Passa o controle para o Scheduler e habilita interrupções, dando vida ao OS.

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::core::handoff::{BootInfo, BOOT_MAGIC};
use alloc::vec::Vec;

/// Função principal do Kernel (High-Level).
///
/// Chamada pelo `_start` (assembly/bare-bones) com a stack já configurada.
/// Esta função **não deve retornar** (o tipo de retorno é `!`).
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Sanity Check (Validação de Sanidade)
    // Garante que não estamos bootando com dados corrompidos ou versão incompatível do Ignite.
    if boot_info.magic != BOOT_MAGIC {
        // Se a magia falhar, não podemos confiar em nada. Travamos a CPU imediatamente.
        Cpu::hang();
    }

    // 1.5. Garantir SSE/FPU inicializado (redundante com _start, mas seguro)
    unsafe {
        crate::arch::x86_64::cpu::X64Cpu::init_sse();
    }

    // 2. Inicializar Sistema de Logs
    // A partir daqui, podemos usar kinfo!, kwarn!, kerror!.
    // O driver serial é inicializado implicitamente na primeira chamada.
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║ Redstone OS Kernel (Forge) - Iniciando ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
    crate::kinfo!("Protocolo de Boot v{}", boot_info.version);

    // 3. Inicializar Arquitetura (HAL)
    // Configura GDT (segmentação) e IDT (tratamento de interrupções/exceções).
    // Crítico fazer isso antes de qualquer operação que possa gerar falhas (ex: acesso a memória inválida).
    crate::kinfo!("(Core) Inicializando hardware (GDT/IDT/TSS)...");
    unsafe {
        crate::arch::platform::gdt::init();
        crate::arch::platform::idt::init();
    }

    #[cfg(feature = "verbose_logs")]
    {
        // Agora que temos GDT/IDT, podemos rodar testes com segurança de que exceções serão capturadas.
        crate::arch::test::run_arch_tests();

        crate::kinfo!("(SelfTest) Validando Core & Bibliotecas...");
        crate::core::test::run_core_tests();
        crate::klib::test::run_klib_tests();
        crate::sys::test::run_sys_tests();
        crate::sync::test::run_sync_tests();
    }

    // 4. Gerenciamento de Memória (PMM, VMM, Heap)
    // Inicializa o alocador de frames físicos, paginação e o Heap do kernel.
    // Habilita o uso de `Box`, `Vec`, `Arc`, etc.
    crate::kinfo!("(Core) Inicializando subsistema de memória...");
    crate::mm::init(boot_info);

    #[cfg(feature = "verbose_logs")]
    {
        crate::kinfo!("(MM) Executando testes de memória...");
        crate::mm::test::run_memory_tests();
    }

    // 5. Drivers Básicos (Hardware Timer & Interrupt Controller)
    // Configura o PIC (Programmable Interrupt Controller) para não conflitar com exceções da CPU
    // e o PIT (Programmable Interval Timer) para gerar o "heartbeat" do scheduler.
    crate::kinfo!("(Core) Configurando controladores de interrupção (PIC/PIT)...");
    unsafe {
        let mut pic = crate::drivers::pic::PICS.lock();
        pic.init();
        pic.unmask(0); // Habilita IRQ0 (Timer)
    }

    {
        let mut pit = crate::drivers::timer::PIT.lock();
        // Configura frequência para 100Hz (10ms por tick).
        let freq = pit.set_frequency(100).expect("Falha ao configurar timer");
        crate::kdebug!("(Core) Timer configurado para {}Hz", freq);
    }

    #[cfg(feature = "verbose_logs")]
    crate::drivers::test::run_driver_tests();

    // 6. Subsistemas Lógicos
    // Inicializa estruturas de IPC (Portas, Mensagens) e Filesystem (VFS).
    crate::ipc::init();
    #[cfg(feature = "verbose_logs")]
    crate::ipc::test::run_ipc_tests();

    crate::fs::init(boot_info);
    #[cfg(feature = "verbose_logs")]
    crate::fs::test::run_fs_tests();

    // Inicializar Vídeo (após memória e antes do console real)
    // Agora inicializamos o CONSOLE, que gerencia o vídeo + texto.
    crate::drivers::console::init_console(boot_info.framebuffer);

    crate::kprintln!("\x1b[36m[Video]\x1b[0m Video ativado"); // Ciano
    crate::kprintln!("\x1b[35m[Console]\x1b[0m Redstone OS v0.1.0 - Console Ativado!"); // Rosa

    // 7. Scheduler (Multitarefa)
    // Inicializa a fila de processos e cria as tarefas iniciais (Kernel Tasks).
    crate::kinfo!("(Core) Ativando escalonador multitarefa...");
    crate::sched::scheduler::init();

    #[cfg(feature = "verbose_logs")]
    {
        crate::sched::test::run_sched_tests();
        crate::security::test::run_security_tests();
        crate::syscall::test::run_syscall_tests();
    }

    // Tenta carregar o processo de usuário (/init)
    spawn_init_process();

    // 8. O Grande Salto (Enable Interrupts)
    // Habilita interrupções (STI). A partir deste ponto, o Timer vai disparar
    // e o Scheduler assumirá o controle da CPU periodicamente.
    crate::kinfo!("Habilitando Interrupções - Sistema Ativo");

    // SAFETY: Tudo está configurado. Habilitar interrupções é seguro e necessário.
    unsafe {
        Cpu::enable_interrupts();
    }

    // Loop da thread "idle" (boot core).
    // Quando não houver nada para fazer, a CPU entra em modo de economia de energia (HLT).
    loop {
        Cpu::halt();
    }
}

/// Tenta localizar e carregar o binário `/init` do sistema de arquivos.
///
/// Se encontrado, cria um processo de usuário (Ring 3).
/// Se não, cria uma tarefa de kernel (dummy) para manter o sistema vivo.
fn spawn_init_process() {
    use crate::fs::vfs::{VfsHandle, ROOT_VFS};

    crate::kinfo!("[Init] spawn_init_process iniciando...");

    // Tenta obter acesso exclusivo ao VFS
    crate::kinfo!("[Init] Obtendo lock do VFS...");
    let vfs = ROOT_VFS.lock();
    crate::kinfo!("[Init] VFS lock OK");

    // Procura pelo arquivo "/system/core/init" (estrutura moderna Redstone)
    if let Ok(node) = vfs.lookup("/system/core/init") {
        crate::kinfo!("(Init) Carregando processo inicial '/system/core/init'...");

        if let Ok(handle) = node.open() {
            let size = node.size() as usize;
            let mut buffer = Vec::with_capacity(size);
            unsafe {
                buffer.set_len(size);
            }

            if let Ok(bytes_read) = handle.read(&mut buffer, 0) {
                crate::kdebug!(
                    "(Init) Arquivo lido ({} bytes), iniciando parsing ELF...",
                    bytes_read
                );
                // Tenta parsear e carregar o ELF na memória
                match unsafe { crate::core::elf::load(&buffer[..bytes_read]) } {
                    Ok(entry_point) => {
                        crate::kdebug!("(Init) ELF carregado em {:#x}", entry_point);

                        // Mapear User Stack (16KB em 0x80000000 - 0x80004000)
                        let user_stack_size = 16 * 1024; // 16KB
                        let user_stack_base = 0x8000_0000 - user_stack_size as u64;
                        let user_stack_top = 0x8000_0000;

                        crate::ktrace!("(Init) Mapeando user stack em {:#x}", user_stack_base);

                        {
                            use crate::mm::pmm::FRAME_ALLOCATOR;
                            use crate::mm::vmm::{self, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};

                            let mut addr = user_stack_base;
                            while addr < user_stack_top {
                                let frame = FRAME_ALLOCATOR
                                    .lock()
                                    .allocate_frame()
                                    .expect("No frames for user stack");
                                unsafe {
                                    vmm::map_page(
                                        addr,
                                        frame.addr,
                                        PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE,
                                    );
                                    // TLB flush
                                    core::arch::asm!("invlpg [{0}]", in(reg) addr, options(nostack, preserves_flags));
                                }
                                addr += 4096;
                            }
                        }
                        crate::kdebug!("(Init) Stack de usuário preparada");

                        // Usa a Page Table atual (CR3) - Em produção, clonaríamos o espaço do kernel.
                        let cr3 = unsafe { crate::arch::platform::memory::cr3() };

                        // Cria a tarefa Ring 3
                        let task =
                            crate::sched::task::Task::new_user(entry_point, user_stack_top, cr3);

                        // Adiciona ao Scheduler
                        crate::sched::scheduler::SCHEDULER.lock().add_task(task);
                        crate::kinfo!("(Init) Processo PID 1 (init) disparado!");
                    }
                    Err(e) => crate::kerror!("(Init) Erro fatal ao carregar ELF: {:?}", e),
                }
            }
        }
    } else {
        // Fallback: Se não houver disco ou /init, roda uma tarefa de teste interna.
        crate::kwarn!("(Init) /init não encontrado via VFS. Criando tarefa dummy de kernel.");
        crate::sched::scheduler::SCHEDULER
            .lock()
            .add_task(crate::sched::task::Task::new_kernel(dummy_init));
    }
}

/// Tarefa de teste (Kernel Mode) para quando não há userspace.
extern "C" fn dummy_init() {
    loop {
        crate::kprint!("A");
        // Spin loop para gastar tempo (simula trabalho)
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}
