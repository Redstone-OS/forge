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
use crate::core::handoff::{BootInfo, BOOT_INFO_VERSION, BOOT_MAGIC};
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

    // Verificar versão do protocolo de boot
    if boot_info.version != BOOT_INFO_VERSION {
        // Versão incompatível - log e halt
        // Nota: Não podemos usar kinfo! ainda pois pode não estar inicializado
        Cpu::hang();
    }

    // 1.5. SSE DESABILITADO
    // SSE foi desabilitado no target spec (x86_64-redstone.json).
    // O compilador não gera instruções SSE, então não precisamos inicializar.

    // 2. Inicializar Sistema de Logs
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║   Kernel Redstone OS (Forge) - v0.0.4  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    crate::kinfo!("Protocolo de Boot v", boot_info.version as u64);

    crate::kok!("Iniciando o Forge v0.0.4");

    // 3. Inicializar Arquitetura (HAL)
    // Configura GDT (segmentação) e IDT (tratamento de interrupções/exceções).
    // Crítico fazer isso antes de qualquer operação que possa gerar falhas (ex: acesso a memória inválida).
    crate::kinfo!("(Core) Inicializando hardware (GDT/IDT/TSS)...");
    unsafe {
        crate::arch::platform::gdt::init();
        crate::arch::platform::idt::init();
    }

    crate::kok!("CPU Inicializada");

    // 4. Gerenciamento de Memória (PMM, VMM, Heap)
    // Inicializa o alocador de frames físicos, paginação e o Heap do kernel.
    // Habilita o uso de `Box`, `Vec`, `Arc`, etc.
    crate::kinfo!("(Core) Inicializando subsistema de memória...");
    unsafe {
        crate::mm::init(boot_info);
    }

    crate::kok!("Memória Inicializada");

    // 5. Drivers Básicos (Hardware Timer & Interrupt Controller)
    // Configura o PIC (Programmable Interrupt Controller) para não conflitar com exceções da CPU
    // e o PIT (Programmable Interval Timer) para gerar o "heartbeat" do scheduler.
    crate::kinfo!("(Core) Configurando controladores de interrupção (PIC/PIT)...");

    // Inicializar PIC (100% ASM)
    crate::drivers::pic::init();
    crate::drivers::pic::unmask(0); // Habilita IRQ0 (Timer)
    crate::kdebug!("(PIC) Inicializado e remapeado para vectors 32-47");

    // Inicializar PIT (100% ASM I/O)
    // Configura frequência para 250Hz (4ms por tick)
    // TODO: Receber a frequência do bootloader
    let freq = crate::drivers::timer::init(250);
    crate::kdebug!("(PIT) Frequência configurada para Hz=", freq as u64);

    crate::kok!("Drivers Inicializados");

    // 6. Subsistemas Lógicos
    // Inicializa estruturas de IPC (Portas, Mensagens) e Filesystem (VFS).
    crate::ipc::init();

    crate::kok!("IPC Inicializado");

    crate::fs::init(boot_info);

    crate::kok!("FS Inicializado");

    // Inicializar Vídeo (framebuffer mínimo)
    // O kernel apenas mapeia o framebuffer e limpa a tela.
    // Console gráfico é responsabilidade do PID1 (userspace).
    unsafe {
        crate::drivers::video::init(&boot_info.framebuffer);
    }

    crate::kok!("Video Inicializado");

    // 7. Scheduler (Multitarefa)
    // Inicializa a fila de processos e cria as tarefas iniciais (Kernel Tasks).
    crate::kinfo!("(Core) Ativando escalonador multitarefa...");
    crate::sched::scheduler::init();

    crate::kok!("Scheduler Inicializado");

    // 8. Syscall (MSRs para instrução syscall)
    // Configura STAR, LSTAR, SFMASK para permitir chamadas de sistema via syscall/sysret.
    crate::syscall::init();

    crate::kok!("Syscall Inicializado");

    // =========================================================================
    // SELF-TESTS: Executados APÓS todos os inits, ANTES do PID1
    // =========================================================================
    // Isso garante que:
    // 1. Todos os subsistemas estão inicializados
    // 2. A stack foi liberada dos inits
    // 3. Se algum teste falhar, o kernel para antes de iniciar userspace
    // =========================================================================
    #[cfg(feature = "self_test")]
    {
        crate::kinfo!("═══════════════════════════════════════");
        crate::kinfo!("        🧪 SELF-TEST");
        crate::kinfo!("═══════════════════════════════════════");

        // Arch tests
        crate::arch::test::run_arch_tests();

        // Core subsystems
        crate::core::test::run_core_tests();
        crate::klib::test::run_klib_tests();
        crate::sys::test::run_sys_tests();
        crate::sync::test::run_sync_tests();

        // Memory subsystem
        crate::mm::test::run_memory_tests();

        // Drivers
        crate::drivers::test::run_driver_tests();

        // Logical subsystems
        crate::ipc::test::run_ipc_tests();
        crate::fs::test::run_fs_tests();

        // Scheduler & Security
        crate::sched::test::run_sched_tests();
        crate::security::test::run_security_tests();
        crate::syscall::test::run_syscall_tests();

        crate::kinfo!("═══════════════════════════════════════");
        crate::kinfo!("    ✅ Todos os testes passaram!");
        crate::kinfo!("═══════════════════════════════════════");
    }

    // Tenta carregar o processo de usuário (/init)
    spawn_init_process();

    // 8. O Grande Salto (Enable Interrupts)
    // Habilita interrupções (STI). A partir deste ponto, o Timer vai disparar
    // e o Scheduler assumirá o controle da CPU periodicamente.
    crate::kok!("Habilitando Interrupções - Sistema Ativo");

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
    use crate::fs::vfs::ROOT_VFS;

    crate::kinfo!("[Init] spawn_init_process iniciando...");

    // Tenta obter acesso exclusivo ao VFS
    crate::kinfo!("[Init] Obtendo acesso ao VFS...");
    let vfs = ROOT_VFS.lock();
    crate::kinfo!("[Init] Acesso ao VFS OK");

    // Procura pelo arquivo "/system/core/init" (estrutura moderna Redstone)
    crate::ktrace!("[Init] [A] Chamando vfs.lookup...");
    let lookup_result = vfs.lookup("/system/core/init");
    crate::ktrace!("[Init] [B] lookup retornou");

    if let Ok(node) = lookup_result {
        crate::ktrace!("[Init] [C] Pattern match OK");
        crate::kinfo!("(Init) Carregando processo inicial '/system/core/init'...");

        if let Ok(handle) = node.open() {
            let size = node.size() as usize;
            let mut buffer = Vec::with_capacity(size);
            unsafe {
                buffer.set_len(size);
            }

            if let Ok(bytes_read) = handle.read(&mut buffer, 0) {
                crate::kdebug!("(Init) Arquivo lido. Tamanho=", bytes_read as u64);
                crate::kdebug!("(Init) Iniciando análise ELF...");
                // Tenta parsear e carregar o ELF na memória
                match unsafe { crate::core::elf::load(&buffer[..bytes_read]) } {
                    Ok(entry_point) => {
                        crate::kdebug!("(Init) ELF carregado em=", entry_point);

                        // Mapear User Stack (16KB em 0x80000000 - 0x80004000)
                        let user_stack_size = 16 * 1024; // 16KB
                        let user_stack_base = 0x8000_0000 - user_stack_size as u64;
                        let user_stack_top = 0x8000_0000;

                        crate::ktrace!("(Init) Mapeando pilha de usuário em=", user_stack_base);

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
                                        frame.addr(),
                                        PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE,
                                    )
                                    .expect("Failed to map user stack");
                                    // TLB flush
                                    core::arch::asm!("invlpg [{0}]", in(reg) addr, options(nostack, preserves_flags));
                                }
                                addr += 4096;
                            }
                        }
                        crate::kdebug!("(Init) Pilha de usuário preparada");

                        // Usa a Page Table atual (CR3) - Em produção, clonaríamos o espaço do kernel.
                        let cr3 = unsafe { crate::arch::platform::memory::cr3() };

                        // Cria a tarefa Ring 3
                        let task =
                            crate::sched::task::Task::new_user(entry_point, user_stack_top, cr3);

                        // Adiciona ao Scheduler
                        crate::sched::scheduler::SCHEDULER.lock().add_task(task);
                        crate::kinfo!("(Init) Processo PID 1 (init) disparado!");
                    }
                    Err(_e) => crate::kerror!("(Init) Erro fatal ao carregar ELF!"),
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
        crate::klog!("A");
        // Spin loop para gastar tempo (simula trabalho)
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}
