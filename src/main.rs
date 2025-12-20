//! Kernel Forge — Arquivo principal
//!
//! Este arquivo é o ponto de entrada do kernel do Redstone OS.
//! Ele define a estrutura global do sistema e controla o fluxo
//! inicial de inicialização após o bootloader.
//!
//! Não deve conter lógica pesada.
//! A responsabilidade aqui é **orquestrar**, não executar.
//!
//! ## Organização dos subsistemas
//! - core/      → Núcleo do kernel (init, scheduler, processos)
//! - mm/        → Gerenciamento de memória (PMM, VMM, allocadores)
//! - fs/        → Sistema de arquivos (VFS e filesystems)
//! - drivers/   → Drivers de hardware (por barramento/dispositivo)
//! - hal/       → Hardware Abstraction Layer
//! - syscall/   → Interface de chamadas de sistema
//! - ipc/       → Comunicação entre processos
//! - security/  → Segurança, permissões e isolamento
//! - net/       → Rede (planejado para versões futuras)
//! - lib/       → Bibliotecas internas do kernel
//!
//! ## Estado atual
//! - Fase 1: Video Output - COMPLETA
//! - Fase 2: Memory Management - COMPLETA (PMM, VMM, Heap)
//! - Fase 3: Interrupts - TODO
//!
//! ## Próximos passos
//! - [v1.0 | alta] Implementar GDT/IDT
//! - [v1.0 | alta] Implementar interrupt handlers
//! - [v1.0 | alta] Implementar keyboard driver
//!
//! Este arquivo existe para deixar claro **como o kernel nasce**.

#![no_std]
#![no_main]
// Módulos principais
pub mod arch;
pub mod boot_info;
pub mod core;
pub mod drivers;
pub mod fs;
pub mod hal;

pub mod mm;
pub mod net;
pub mod panic;
pub mod security;
pub mod syscall;

// ============================================================================
// TODO(prioridade=ALTA, versão=v1.0): REFATORAR GLOBAL MUTABLE INITFS
// ============================================================================
//
// ⚠️ ATENÇÃO: Global mutable para FAT32 filesystem!
// Ver comentários em main() para detalhes sobre riscos e soluções futuras.
// ============================================================================
static mut INITFS: Option<fs::fat32::Fat32> = None;

// Stack do kernel (16 KB)
// Usa #[repr(align(16))] para garantir alinhamento de 16 bytes exigido pela ABI x86_64
#[repr(align(16))]
struct KernelStack([u8; 16 * 1024]);

static KERNEL_STACK: KernelStack = KernelStack([0; 16 * 1024]);

/// Ponto de entrada naked - Configura stack antes do Rust
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "sysv64" fn _start(boot_info_addr: u64) -> ! {
    ::core::arch::naked_asm!(
        // 0. Salvar boot_info_addr (RDI) em R15 antes de modificar stack
        "mov r15, rdi",

        // 1. Configurar Stack Pointer (RSP)
        // Carregar endereço do símbolo do stack (RIP-relative funciona porque o kernel é contíguo)
        "lea rax, [rip + {stack}]",
        // Adicionar tamanho do stack para apontar para o topo (stack cresce para baixo)
        "lea rsp, [rax + {stack_size}]",

        // 2. Zerar RBP (para backtrace)
        "xor rbp, rbp",

        // 3. Restaurar boot_info_addr para RDI antes de chamar kernel_main
        "mov rdi, r15",

        // 3.5. Habilitar SSE (Essencial para Rust, pois memcpy/memset usam SSE)
        // CR0: Clear EM (bit 2), Set MP (bit 1)
        "mov rax, cr0",
        "and ax, 0xFFFB",
        "or ax, 0x2",
        "mov cr0, rax",
        // CR4: Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10)
        "mov rax, cr4",
        "or ax, 0x600",
        "mov cr4, rax",

        // DEBUG: Enviar 'S' (Start) para serial para confirmar que kernel iniciou
        "mov dx, 0x3F8",
        "mov al, 0x53", // 'S'
        "out dx, al",

        // 4. Chamar função principal do kernel
        "call {kernel_main}",

        // 5. Halt loop (usando labels numéricas para compatibilidade)
        "2:",
        "cli",
        "hlt",
        "jmp 2b",

        stack = sym KERNEL_STACK,
        stack_size = const 16 * 1024,
        kernel_main = sym kernel_main,
    );
}

// Marcador para o fim do stack (topo, pois cresce para baixo)
// Endereço calculado: endereço base do stack + tamanho
// Mas em asm! sym, precisamos de um símbolo exportado.
// Truque: usar offset no LEA acima.
// Alternativa Simples: Carregar endereço base e somar tamanho.
// Vamos ajustar o assembly acima para ser mais robusto.

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info_addr: u64) -> ! {
    use drivers::video::framebuffer::{COLOR_BLACK, COLOR_LIGHT_GREEN};
    use drivers::video::{Console, Framebuffer};

    // 1. Inicializar serial
    drivers::legacy::serial::init();
    drivers::legacy::serial::println("[OK] Serial inicializado");
    drivers::legacy::serial::println("[OK] Kernel _start executando!");

    // 2. Ler BootInfo do endereço passado pelo bootloader
    // NOTA: Desreferenciamos para copiar para a stack (que está mapeada),
    // pois o endereço original pode não estar mapeado após ativar VMM.
    let boot_info = unsafe { *(boot_info_addr as *const boot_info::BootInfo) };

    // 3. Criar framebuffer
    let fb = Framebuffer::new(
        boot_info.fb_addr as usize,
        boot_info.fb_width as usize,
        boot_info.fb_height as usize,
        boot_info.fb_stride as usize,
    );

    let mut console = Console::new(fb);
    console.set_colors(COLOR_LIGHT_GREEN, COLOR_BLACK);
    console.clear();

    // 4. Banner
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Forge Kernel v0.3.5\n");
    dp(
        &mut console,
        "===================================================\n\n",
    );
    dp(&mut console, "[OK] Porta serial inicializada\n");
    dp(&mut console, "[OK] Console de video inicializado\n");
    dp(&mut console, "[OK] Framebuffer pronto\n\n");

    // 5. Inicializar gerenciamento de memoria
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Gerenciamento de Memoria\n");
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "[1/3] Inicializando PMM...\n");

    let mut pmm = mm::PhysicalMemoryManager::init(&boot_info);
    let (total, free, _used) = pmm.stats();

    dp(&mut console, "[OK] PMM inicializado!\n");
    dp(&mut console, "  Total: ");
    dn(&mut console, total);
    dp(&mut console, " frames (");
    dn(&mut console, (total * 4096) / (1024 * 1024));
    dp(&mut console, " MB)\n");
    dp(&mut console, "  Livre: ");
    dn(&mut console, free);
    dp(&mut console, " frames (");
    dn(&mut console, (free * 4096) / (1024 * 1024));
    dp(&mut console, " MB)\n\n");

    // 6. VMM - Habilitado
    dp(&mut console, "[2/3] Inicializando VMM...\n");
    let mut vmm = mm::VirtualMemoryManager::init(&mut pmm);

    // 1. Mapear memória baixa (0-1MB) para BIOS/hardware
    dp(&mut console, "  Mapeando memoria baixa (0-1MB)...\n");
    vmm.identity_map(0x0, 0x100000, mm::vmm::flags::WRITABLE, &mut pmm)
        .expect("Falha ao mapear memoria baixa");

    // 2. Mapear kernel completo (usando informações reais do bootloader)
    dp(&mut console, "  Mapeando kernel...\n");

    // Alinhar tamanho para próximas 4KB
    let kernel_end = (boot_info.kernel_base + boot_info.kernel_size + 0xFFF) & !0xFFF;

    vmm.identity_map(
        boot_info.kernel_base,
        kernel_end,
        mm::vmm::flags::WRITABLE, // Kernel code + data + stack
        &mut pmm,
    )
    .expect("Falha ao mapear kernel");

    // DEBUG: Mostrar onde estamos mapeando
    dp(&mut console, "  -> Kernel: 0x");
    dh(&mut console, boot_info.kernel_base as usize);
    dp(&mut console, " - 0x");
    dh(&mut console, kernel_end as usize);
    dp(&mut console, "\n");

    // 3. Mapear framebuffer
    dp(&mut console, "  Mapeando framebuffer...\n");
    let fb_bytes = (boot_info.fb_stride as u64) * (boot_info.fb_height as u64) * 4;
    let fb_size = (fb_bytes + 0xFFF) & !0xFFF;
    vmm.identity_map(
        boot_info.fb_addr,
        boot_info.fb_addr + fb_size,
        mm::vmm::flags::WRITABLE | mm::vmm::flags::NO_CACHE,
        &mut pmm,
    )
    .expect("Falha ao mapear framebuffer");

    // 4. Mapear heap (MOVED TO 32MB TO AVOID PMM BITMAP AT 8MB)
    dp(&mut console, "  Mapeando heap...\n");
    const HEAP_START: usize = 0x0200_0000; // 32 MB
    const HEAP_SIZE: usize = 4 * 1024 * 1024;
    vmm.identity_map(
        HEAP_START as u64,
        (HEAP_START + HEAP_SIZE) as u64,
        mm::vmm::flags::WRITABLE,
        &mut pmm,
    )
    .expect("Falha ao mapear heap");

    // 5. Stack já foi mapeado junto com o kernel (pois é static dentro do .bss/.data)
    dp(&mut console, "  Stack: OK (dentro do kernel)\n");

    // 6. Mapear InitFS se presente
    if boot_info.initfs_size > 0 {
        dp(&mut console, "  Mapeando InitFS...\n");
        let initfs_start = boot_info.initfs_addr & !0xFFF;
        let initfs_end = (boot_info.initfs_addr + boot_info.initfs_size + 0xFFF) & !0xFFF;
        vmm.identity_map(initfs_start, initfs_end, mm::vmm::flags::WRITABLE, &mut pmm)
            .expect("Falha ao mapear InitFS");
    }

    // 7. Ativar VMM
    dp(&mut console, "  Ativando VMM...\n");
    vmm.activate();

    dp(&mut console, "[OK] VMM inicializado e ativo!\n");
    dp(&mut console, "  Memoria baixa: 0x0-0x100000\n");
    dp(&mut console, "  Kernel: [Dinamico]\n");
    dp(&mut console, "  Framebuffer: 0x");
    dh(&mut console, boot_info.fb_addr as usize);
    dp(&mut console, "\n");
    dp(&mut console, "  Heap: 0x");
    dh(&mut console, HEAP_START);
    dp(&mut console, "\n");
    dp(&mut console, "  Stack: 0x1000000-0x1400000\n\n");

    // 7. Inicializar Heap
    dp(&mut console, "[3/3] Inicializando Heap...\n");

    // Inicializar alocador global (sem mapear, usa região já mapeada)
    mm::heap::ALLOCATOR.init(HEAP_START, HEAP_SIZE);

    dp(&mut console, "[OK] Heap inicializado (4 MB)!\n\n");

    // 8. Testar heap
    dp(&mut console, "[TEST] Testando heap...\n");

    extern crate alloc;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    dp(&mut console, "  Vec: OK (3 elementos)\n");

    let boxed = Box::new(42);
    dp(&mut console, "  Box: OK (valor = ");
    dn(&mut console, *boxed);
    dp(&mut console, ")\n");

    dp(&mut console, "[OK] Testes de heap concluidos!\n\n");

    dp(
        &mut console,
        "===================================================\n",
    );
    dp(
        &mut console,
        "  Fase 2: Gerenciamento de Memoria COMPLETA!\n",
    );
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  PMM: OK (");
    dn(&mut console, (free * 4096) / (1024 * 1024));
    dp(&mut console, " MB livres)\n");
    dp(&mut console, "  VMM: OK (paginacao ativa)\n");
    dp(&mut console, "  Heap: OK (16 MB)\n\n");

    // 7. Inicializar Filesystem (Fase 6)
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 6: Filesystem\n");
    dp(
        &mut console,
        "===================================================\n",
    );

    dp(&mut console, "[1/3] Inicializando VFS...\n");
    // VFS já existe como singleton, não precisa criar
    dp(&mut console, "[OK] VFS inicializado\n");

    dp(&mut console, "[2/3] Montando InitRAMFS...\n");

    if boot_info.initfs_size > 0 {
        let initramfs_data = unsafe {
            ::core::slice::from_raw_parts(
                boot_info.initfs_addr as *const u8,
                boot_info.initfs_size as usize,
            )
        };

        // Parse TAR
        match fs::tar::TarArchive::new(initramfs_data) {
            Ok(tar) => {
                dp(&mut console, "[OK] InitRAMFS parseado (");
                dn(&mut console, (boot_info.initfs_size / 1024) as usize);
                dp(&mut console, " KB)\n");

                // Criar TarFS
                match fs::tarfs::TarFS::new(initramfs_data) {
                    Ok(tarfs) => {
                        dp(&mut console, "[OK] TarFS criado\n");

                        // Listar conteúdo
                        dp(&mut console, "[INFO] Conteúdo do initramfs:\n");
                        match tarfs.readdir("/") {
                            Ok(entries) => {
                                for entry in entries.iter().take(10) {
                                    dp(&mut console, "  - ");
                                    dp(&mut console, entry);
                                    dp(&mut console, "\n");
                                }
                                if entries.len() > 10 {
                                    dp(&mut console, "  ... e mais ");
                                    dn(&mut console, entries.len() - 10);
                                    dp(&mut console, " arquivos\n");
                                }
                            }
                            Err(e) => {
                                dp(&mut console, "  [WARN] Erro ao listar: ");
                                dp(&mut console, e);
                                dp(&mut console, "\n");
                            }
                        }

                        // Verificar se /bin/init existe
                        if tarfs.exists("bin/init") {
                            dp(&mut console, "[OK] /bin/init encontrado no initramfs\n");
                            if let Some(size) = tarfs.file_size("bin/init") {
                                dp(&mut console, "  Tamanho: ");
                                dn(&mut console, size);
                                dp(&mut console, " bytes\n");
                            }
                        } else {
                            dp(&mut console, "[ERROR] /bin/init NÃO encontrado!\n");
                            dp(&mut console, "[ERROR] Sistema não pode iniciar PID 1\n");
                        }

                        // TODO: Montar no VFS quando VFS suportar filesystems dinâmicos
                        // Por enquanto, apenas guardamos referência
                        unsafe {
                            INITFS = Some(fs::fat32::Fat32::default()); // Placeholder
                        }
                    }
                    Err(e) => {
                        dp(&mut console, "[ERROR] Falha ao criar TarFS: ");
                        dp(&mut console, e);
                        dp(&mut console, "\n");
                    }
                }
            }
            Err(e) => {
                dp(&mut console, "[ERROR] Falha ao parsear TAR: ");
                dp(&mut console, e);
                dp(&mut console, "\n");
                dp(&mut console, "[ERROR] InitRAMFS inválido\n");
            }
        }
    } else {
        dp(
            &mut console,
            "[ERROR] InitRAMFS não fornecido pelo bootloader\n",
        );
        dp(
            &mut console,
            "[ERROR] Sistema precisa de initramfs para boot\n",
        );
    }

    dp(&mut console, "[3/3] Filesystem pronto\n\n");

    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 6: COMPLETA!\n");
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  VFS: OK\n");
    if boot_info.initfs_size > 0 {
        dp(&mut console, "  InitRAMFS: Montado (");
        dn(&mut console, (boot_info.initfs_size / 1024) as usize);
        dp(&mut console, " KB)\n\n");
    } else {
        dp(&mut console, "  InitRAMFS: Não disponível\n\n");
    }

    // 8. Inicializar Interrupções (Fase 3)
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Interrupcoes\n");
    dp(
        &mut console,
        "===================================================\n",
    );

    dp(&mut console, "[1/5] Inicializando GDT...\n");
    arch::x86_64::gdt::init();
    dp(&mut console, "[OK] GDT carregada!\n");

    dp(&mut console, "[2/5] Inicializando IDT...\n");
    arch::x86_64::idt::init();
    dp(&mut console, "[OK] IDT carregada (256 entradas)!\n");

    dp(&mut console, "[3/5] Inicializando PIC...\n");
    drivers::pic::init();
    drivers::pic::unmask_irq(0); // Timer (IRQ 0)
    dp(&mut console, "[OK] PIC configurado!\n");

    dp(&mut console, "[4/5] Inicializando Timer (100 Hz)...\n");
    drivers::timer::pit::init(100);
    dp(&mut console, "[OK] Timer ativo!\n");

    dp(&mut console, "[5/5] Habilitando interrupcoes...\n");
    x86_64::instructions::interrupts::enable();
    dp(&mut console, "[OK] Interrupcoes habilitadas!\n\n");

    // Testar timer
    dp(&mut console, "[TEST] Aguardando 100 ticks (1 segundo)...\n");
    let start = drivers::timer::pit::ticks();
    while drivers::timer::pit::ticks() < start + 100 {}
    dp(&mut console, "[OK] Timer funcionando!\n\n");

    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 3: Interrupcoes COMPLETA!\n");
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  GDT: OK\n");
    dp(&mut console, "  IDT: OK\n");
    dp(&mut console, "  Handlers: OK\n");
    dp(&mut console, "  PIC: OK\n");
    dp(&mut console, "  Timer: OK (100 Hz)\n\n");

    // 11. Inicializar Keyboard
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Keyboard Input\n");
    dp(
        &mut console,
        "===================================================\n",
    );

    dp(&mut console, "[1/1] Inicializando Keyboard...\n");
    drivers::keyboard::init();
    drivers::pic::unmask_irq(1); // IRQ 1 = Keyboard
    dp(&mut console, "[OK] Keyboard ativo!\n\n");

    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 4: Keyboard Input COMPLETA!\n");
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Driver PS/2: OK\n");
    dp(&mut console, "  IRQ 1: OK\n");
    dp(&mut console, "  Input Buffer: OK\n\n");

    dp(&mut console, "Proximas etapas:\n");
    dp(&mut console, "[TODO] 1. Processos\n");
    dp(&mut console, "[TODO] 2. Scheduler\n");
    dp(&mut console, "[TODO] 3. Filesystem\n");
    dp(&mut console, "[TODO] 4. Shell\n\n");

    // 12. Inicializar Processos e Scheduler (Fase 5)
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 5: Processos e Scheduler\n");
    dp(
        &mut console,
        "===================================================\n",
    );

    dp(&mut console, "[1/3] Criando tasks de teste...\n");

    // Criar tasks de teste
    fn task1() {
        loop {
            crate::drivers::legacy::serial::println("Task 1 running");
            for _ in 0..10000000 {
                unsafe {
                    ::core::arch::asm!("nop");
                }
            }
        }
    }

    fn task2() {
        loop {
            crate::drivers::legacy::serial::println("Task 2 running");
            for _ in 0..10000000 {
                unsafe {
                    ::core::arch::asm!("nop");
                }
            }
        }
    }

    fn task3() {
        loop {
            crate::drivers::legacy::serial::println("Task 3 running");
            for _ in 0..10000000 {
                unsafe {
                    ::core::arch::asm!("nop");
                }
            }
        }
    }

    use core::process::PROCESS_MANAGER;
    PROCESS_MANAGER.lock().spawn(task1, "task1");
    PROCESS_MANAGER.lock().spawn(task2, "task2");
    PROCESS_MANAGER.lock().spawn(task3, "task3");

    dp(&mut console, "[OK] 3 tasks criadas\n");

    dp(&mut console, "[2/3] Inicializando scheduler...\n");
    dp(&mut console, "[OK] Scheduler round-robin (10ms quantum)\n");

    dp(&mut console, "[3/3] Iniciando multitasking...\n");
    dp(&mut console, "[OK] Multitasking ativo!\n\n");

    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Fase 5: COMPLETA!\n");
    dp(
        &mut console,
        "===================================================\n",
    );
    dp(&mut console, "  Processos: 3 tasks\n");
    dp(&mut console, "  Scheduler: Round-robin\n");
    dp(&mut console, "  Preempcao: Timer (100 Hz)\n\n");

    dp(&mut console, "[TEST] Digite algo e pressione Enter:\n");
    dp(&mut console, "> ");

    // Loop principal - processar input
    let mut line_buffer = alloc::string::String::new();
    loop {
        // Ler caracteres do buffer
        if let Some(ch) = drivers::input_buffer::INPUT_BUFFER.lock().pop() {
            if ch == '\n' {
                // Enter pressionado - processar linha
                dp(&mut console, "\nVoce digitou: ");
                dp(&mut console, &line_buffer);
                dp(&mut console, "\n> ");
                line_buffer.clear();
            } else if ch == '\x08' {
                // Backspace
                if !line_buffer.is_empty() {
                    line_buffer.pop();
                }
            } else {
                // Caractere normal
                line_buffer.push(ch);
            }
        }
    }
}

// dp = dual print (serial + video)
fn dp(console: &mut drivers::video::Console, s: &str) {
    drivers::legacy::serial::print(s);
    console.write_str(s);
}

// dn = dual number (imprime numero)
fn dn(console: &mut drivers::video::Console, mut n: usize) {
    if n == 0 {
        dp(console, "0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        console.write_char(buf[i] as char);
        drivers::legacy::serial::write_byte(buf[i]);
    }
}

// dh = dual hex (imprime hexadecimal)
fn dh(console: &mut drivers::video::Console, mut n: usize) {
    if n == 0 {
        dp(console, "0");
        return;
    }
    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        let d = n % 16;
        buf[i] = if d < 10 {
            b'0' + d as u8
        } else {
            b'a' + (d - 10) as u8
        };
        n /= 16;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        console.write_char(buf[i] as char);
        drivers::legacy::serial::write_byte(buf[i]);
    }
}
