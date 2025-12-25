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
use alloc::sync::Arc;
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

    // 2. Inicializar Sistema de Logs
    // A partir daqui, podemos usar kinfo!, kwarn!, kerror!.
    // O driver serial é inicializado implicitamente na primeira chamada.
    crate::kinfo!("Redstone OS Kernel (Forge) - Iniciando");
    crate::kinfo!("Protocolo de Boot v{}", boot_info.version);

    // 3. Inicializar Arquitetura (HAL)
    // Configura GDT (segmentação) e IDT (tratamento de interrupções/exceções).
    // Crítico fazer isso antes de qualquer operação que possa gerar falhas (ex: acesso a memória inválida).
    crate::kinfo!("Inicializando Arquitetura (GDT/IDT/TSS)...");
    unsafe {
        crate::arch::platform::gdt::init();
        crate::arch::platform::idt::init();
    }

    // 4. Gerenciamento de Memória (PMM, VMM, Heap)
    // Inicializa o alocador de frames físicos, paginação e o Heap do kernel.
    // Habilita o uso de `Box`, `Vec`, `Arc`, etc.
    crate::kinfo!("Inicializando Memória (PMM/VMM/Heap)...");
    crate::mm::init(boot_info);

    // 5. Drivers Básicos (Hardware Timer & Interrupt Controller)
    // Configura o PIC (Programmable Interrupt Controller) para não conflitar com exceções da CPU
    // e o PIT (Programmable Interval Timer) para gerar o "heartbeat" do scheduler.
    crate::kinfo!("Configurando Drivers (PIC/PIT)...");
    unsafe {
        let mut pic = crate::drivers::pic::PICS.lock();
        pic.init();
        pic.unmask(0); // Habilita IRQ0 (Timer)
                       // pic.unmask(1); // Futuro: Habilitar IRQ1 (Teclado)
    }

    {
        let mut pit = crate::drivers::timer::PIT.lock();
        // Configura frequência para 100Hz (10ms por tick).
        // Usamos expect pois sem timer o sistema não pode operar em multitarefa.
        let freq = pit.set_frequency(100).expect("Falha ao configurar timer");
        crate::kinfo!("Timer configurado para {}Hz", freq);
    }

    // 6. Subsistemas Lógicos
    // Inicializa estruturas de IPC (Portas, Mensagens) e Filesystem (VFS).
    crate::ipc::init();
    crate::fs::init(boot_info);

    // 7. Scheduler (Multitarefa)
    // Inicializa a fila de processos e cria as tarefas iniciais (Kernel Tasks).
    crate::kinfo!("Inicializando Scheduler...");
    crate::sched::scheduler::init();

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
    crate::kinfo!("[Init] lookup /system/core/init...");
    if let Ok(node) = vfs.lookup("/system/core/init") {
        crate::kinfo!("[Init] Found /system/core/init, loading ELF...");

        if let Ok(handle) = node.open() {
            let size = node.size() as usize;
            // Aloca buffer para ler o executável inteiro
            let mut buffer = Vec::with_capacity(size);
            unsafe {
                buffer.set_len(size);
            }

            if let Ok(bytes_read) = handle.read(&mut buffer, 0) {
                // Tenta parsear e carregar o ELF na memória
                match unsafe { crate::core::elf::load(&buffer[..bytes_read]) } {
                    Ok(entry_point) => {
                        crate::kinfo!("[Init] ELF carregado. Ponto de entrada: {:#x}", entry_point);

                        // Define onde será o topo da stack do usuário (arbitrário por enquanto)
                        let user_stack_top = 0x8000_0000;

                        // Usa a Page Table atual (CR3) - Em produção, clonaríamos o espaço do kernel.
                        let cr3 = unsafe { crate::arch::platform::memory::cr3() };

                        // Cria a tarefa Ring 3
                        let task =
                            crate::sched::task::Task::new_user(entry_point, user_stack_top, cr3);

                        // Adiciona ao Scheduler
                        crate::sched::scheduler::SCHEDULER.lock().add_task(task);
                        crate::kinfo!("[Init] Processo PID 1 iniciado!");
                    }
                    Err(e) => crate::kerror!("[Init] Falha ao carregar ELF: {:?}", e),
                }
            }
        }
    } else {
        // Fallback: Se não houver disco ou /init, roda uma tarefa de teste interna.
        crate::kwarn!("[Init] /init não encontrado via VFS. Criando tarefa dummy de kernel.");
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
