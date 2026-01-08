//! # SMP Bringup
//!
//! Acordar CPUs secundárias (Application Processors - APs).
//!
//! ## Sequência INIT-SIPI-SIPI
//!
//! ```text
//! BSP                                    AP
//!  │                                      │
//!  │  ┌─────────────────────────────────┐ │
//!  │  │ 1. Preparar Trampoline Code     │ │
//!  │  │    - Copiar para < 1MB          │ │
//!  │  │    - Configurar stack           │ │
//!  └─────────────────────────────────────┘ │
//!  │                                      │
//!  ├──────── INIT IPI ───────────────────►│ (Reset AP)
//!  │                                      │ Espera 10ms
//!  ├──────── SIPI #1 ────────────────────►│ (Startup)
//!  │                                      │ Espera 200µs
//!  ├──────── SIPI #2 ────────────────────►│ (Retry)
//!  │                                      │
//!  │                                      ▼
//!  │  ┌─────────────────────────────────┐
//!  │  │ AP executa trampoline:          │
//!  │  │  - 16-bit real mode             │
//!  │  │  - Habilita protected mode      │
//!  │  │  - Habilita long mode           │
//!  │  │  - Salta para ap_entry()        │
//!  │  └─────────────────────────────────┘
//!  │                                      │
//!  │◄────────── Sinaliza Ready ───────────┤
//!  │                                      │
//! ```

use crate::arch::x86_64::apic::lapic;
use crate::core::smp::topology;
use crate::core::smp::trampoline::{trampoline_code, TRAMPOLINE_ADDR};

/// Tamanho da stack por AP (64KB)
pub const AP_STACK_SIZE: usize = 64 * 1024;

/// Número máximo de APs suportados
pub const MAX_APS: usize = 63;

/// Timeout para esperar AP acordar (em iterações)
const AP_STARTUP_TIMEOUT: u32 = 1_000_000;

/// Flag global para APs sinalizarem que acordaram
/// Cada bit representa um AP (bit N = AP com logical_id N)
static AP_READY_FLAGS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Stacks para os APs (alocados estaticamente por simplicidade)
/// Em produção, alocar dinamicamente do heap
#[repr(C, align(4096))]
struct ApStacks([[u8; AP_STACK_SIZE]; MAX_APS]);

static mut AP_STACKS: ApStacks = ApStacks([[0; AP_STACK_SIZE]; MAX_APS]);

/// Inicializa o subsistema de SMP bringup
pub fn init() {
    crate::kinfo!("(SMP/Bringup) Inicializado");
}

/// Acorda todos os APs
///
/// # Safety
///
/// - A topologia deve estar inicializada
/// - O LAPIC do BSP deve estar habilitado
/// - As page tables devem estar configuradas
pub unsafe fn bringup_all_aps() {
    let topo = topology::get();
    let cpu_count = topo.count();

    if cpu_count <= 1 {
        crate::kinfo!("(SMP/Bringup) Sistema single-core, pulando bringup");
        return;
    }

    crate::kinfo!("(SMP/Bringup) Acordando", (cpu_count - 1) as u64, "APs...");

    // Preparar trampoline
    prepare_trampoline();

    // Acordar cada AP
    let mut _success_count = 0u32;
    for cpu in topo.iter_aps() {
        crate::kdebug!("(SMP/Bringup) Acordando AP", cpu.apic_id as u64);

        // Configurar dados específicos deste AP
        setup_ap_data(cpu.logical_id, cpu.apic_id);

        match wake_ap(cpu.apic_id, cpu.logical_id) {
            Ok(()) => {
                crate::kdebug!("(SMP/Bringup) AP", cpu.logical_id as u64, "acordado");
                _success_count += 1;
            }
            Err(e) => {
                crate::kerror!("(SMP/Bringup) Falha ao acordar AP", cpu.apic_id as u64);
                crate::kerror!("  Erro:", e);
            }
        }
    }

    // Contar quantos APs realmente acordaram
    let ready = AP_READY_FLAGS.load(core::sync::atomic::Ordering::Acquire);
    let ready_count = ready.count_ones();
    crate::kinfo!(
        "(SMP/Bringup)",
        ready_count as u64,
        "de",
        (cpu_count - 1) as u64,
        "APs prontos"
    );
}

/// Prepara o código trampoline na memória baixa
unsafe fn prepare_trampoline() {
    crate::kdebug!("(SMP/Bringup) Preparando trampoline em", TRAMPOLINE_ADDR);

    let trampoline_dst = TRAMPOLINE_ADDR as *mut u8;
    // Limpar toda a área do trampoline primeiro (código + dados)
    core::ptr::write_bytes(trampoline_dst, 0, 0x200); // 512 bytes total

    // Copiar código binário do trampoline
    let code = trampoline_code();
    let code_size = code.len();
    core::ptr::copy_nonoverlapping(code.as_ptr(), trampoline_dst, code_size);

    crate::kdebug!(
        "(SMP/Bringup) Trampoline copiado,",
        code_size as u64,
        "bytes"
    );
}

/// Configura dados específicos para um AP
unsafe fn setup_ap_data(logical_id: u32, _apic_id: u32) {
    let data = crate::core::smp::trampoline::get_trampoline_data();

    // CR3 atual (page table do kernel)
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3);
    (*data).cr3 = cr3;

    // GDT do kernel
    #[repr(C, packed)]
    struct GdtPtr {
        limit: u16,
        base: u64,
    }
    let mut gdtr = GdtPtr { limit: 0, base: 0 };
    core::arch::asm!("sgdt [{}]", in(reg) &mut gdtr, options(nostack));
    (*data).gdt_base = gdtr.base;
    (*data).gdt_limit = gdtr.limit;

    // Stack para este AP
    let stack_idx = (logical_id - 1) as usize; // BSP é 0, APs começam em 1
    if stack_idx < MAX_APS {
        let stack_base = &AP_STACKS.0[stack_idx] as *const [u8; AP_STACK_SIZE] as u64;
        (*data).stack_top = stack_base + AP_STACK_SIZE as u64;
    } else {
        crate::kerror!(
            "(SMP/Bringup) AP index",
            logical_id as u64,
            "excede MAX_APS!"
        );
        (*data).stack_top = 0;
    }

    // ID lógico
    (*data).ap_id = logical_id;

    // Entry point Rust
    (*data).rust_entry = ap_entry as u64;

    crate::kdebug!(
        "(SMP/Bringup) AP",
        logical_id as u64,
        "stack:",
        (*data).stack_top
    );
}

/// Acorda um AP específico
///
/// # Argumentos
///
/// - `apic_id`: APIC ID de hardware do AP
/// - `logical_id`: ID lógico (0 a N-1)
unsafe fn wake_ap(apic_id: u32, logical_id: u32) -> Result<(), &'static str> {
    crate::kdebug!(
        "(SMP/Bringup) Enviando INIT IPI para APIC ID",
        apic_id as u64
    );

    // 1. Enviar INIT IPI
    lapic::send_init_ipi(apic_id);
    crate::kdebug!("(SMP/Bringup) INIT IPI enviado, aguardando 10ms");

    // 2. Esperar 10ms
    delay_ms(10);

    // 3. Enviar primeiro SIPI
    let sipi_vector = (TRAMPOLINE_ADDR >> 12) as u8;
    crate::kdebug!("(SMP/Bringup) Enviando SIPI vetor:", sipi_vector as u64);
    lapic::send_sipi(apic_id, sipi_vector);

    // 4. Esperar 1ms para o AP processar
    delay_ms(1);

    // 5. Verificar se acordou
    crate::kdebug!("(SMP/Bringup) Verificando se AP respondeu...");
    if !wait_for_ap(logical_id, AP_STARTUP_TIMEOUT) {
        // 6. Enviar segundo SIPI (retry)
        crate::kdebug!("(SMP/Bringup) Reenviando SIPI...");
        lapic::send_sipi(apic_id, sipi_vector);
        delay_ms(5);

        // 7. Esperar com timeout maior
        if !wait_for_ap(logical_id, AP_STARTUP_TIMEOUT * 10) {
            crate::kerror!("(SMP/Bringup) AP", apic_id as u64, "timeout!");
            return Err("AP não respondeu após SIPI");
        }
    }

    // Marcar como online na topologia
    topology::get_mut().set_online(logical_id as usize);
    crate::kdebug!("(SMP/Bringup) AP", logical_id as u64, "online!");

    Ok(())
}

/// Verifica se um AP está pronto
fn check_ap_ready(logical_id: u32) -> bool {
    let flags = AP_READY_FLAGS.load(core::sync::atomic::Ordering::Acquire);
    (flags & (1 << logical_id)) != 0
}

/// Espera um AP ficar pronto com timeout
fn wait_for_ap(logical_id: u32, timeout: u32) -> bool {
    for _ in 0..timeout {
        if check_ap_ready(logical_id) {
            return true;
        }
        // Pequeno delay
        core::hint::spin_loop();
    }
    false
}

/// Chamado pelo AP quando acorda e está pronto
///
/// # Safety
///
/// Deve ser chamado apenas pelo código do trampoline do AP.
#[no_mangle]
pub extern "C" fn ap_signal_ready(logical_id: u32) {
    AP_READY_FLAGS.fetch_or(1 << logical_id, core::sync::atomic::Ordering::Release);
}

// =============================================================================
// Delay Helpers
// =============================================================================

/// Delay aproximado em milissegundos (busy-wait)
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 1000) {
        delay_us(1);
    }
}

/// Delay aproximado em microsegundos (busy-wait)
fn delay_us(us: u32) {
    // Aproximação baseada em ~1000 iterações por µs em CPUs modernas
    for _ in 0..(us * 100) {
        core::hint::spin_loop();
    }
}

// =============================================================================
// AP Entry Point
// =============================================================================

/// Ponto de entrada Rust para APs
///
/// Chamado pelo trampoline após entrar em long mode.
/// Inicializa estruturas per-CPU e entra no loop do scheduler.
///
/// ## Fluxo
///
/// ```text
/// 1. LAPIC init
/// 2. Obter logical_id via topologia
/// 3. Inicializar estrutura per-CPU
/// 4. Criar idle task para este AP
/// 5. Sinalizar ready para BSP
/// 6. Entrar no loop do scheduler (nunca retorna)
/// ```
#[no_mangle]
pub extern "C" fn ap_entry(_logical_id: u32) -> ! {
    // 1. Ler nosso APIC ID diretamente do hardware
    // ANTES de carregar GDT/TSS porque precisamos saber qual CPU somos
    let apic_id = lapic::current_apic_id();

    // 2. Buscar logical_id na topologia pelo APIC ID
    let logical_id = {
        let topo = topology::get();
        if let Some(cpu) = topo.find_by_apic_id(apic_id) {
            cpu.logical_id
        } else {
            0 // Fallback (não deveria acontecer)
        }
    };

    // 3. Carregar GDT do kernel COM TSS próprio para esta CPU
    // O trampoline usa uma GDT temporária com seletores diferentes, precisamos
    // carregar a GDT do kernel para que os handlers de interrupção funcionem.
    // Cada CPU tem seu próprio TSS para evitar conflito do flag "busy".
    unsafe {
        crate::arch::x86_64::gdt::init_ap(logical_id as usize);
    }

    // 4. Habilitar SSE/FPU para esta CPU
    // O trampoline não habilita SSE, então precisamos fazer aqui.
    // Sem isso, processos que usam instruções SSE/SIMD falharão com #UD.
    unsafe {
        crate::arch::x86_64::cpu::Cpu::enable_sse();
    }

    // 5. Carregar a IDT (com seletores e TSS corretos agora)
    unsafe {
        crate::arch::x86_64::idt::IDT.load();
    }

    // 6. Inicializar MSRs de Syscall para esta CPU
    // CRÍTICO: Sem isso, a instrução `syscall` causa #UD (Invalid Opcode)
    // porque EFER.SCE não está habilitado e LSTAR/STAR/FMASK não estão configurados.
    unsafe {
        crate::arch::x86_64::syscall::init();
    }

    // 7. Inicializar LAPIC local
    unsafe {
        lapic::init();
    }

    crate::kdebug!(
        "(SMP/AP)",
        logical_id as u64,
        "iniciado! APIC:",
        apic_id as u64
    );

    // 6. Inicializar estrutura per-CPU para este AP
    crate::sched::core::per_cpu::init_cpu(logical_id as usize);

    // 6. Criar idle task para este AP
    crate::sched::core::idle::init_idle_task_for_cpu(logical_id as usize);

    // 7. Sinalizar que acordou (BSP está esperando)
    ap_signal_ready(logical_id);

    // 8. Entrar no loop do scheduler (nunca retorna)
    crate::kinfo!("(SMP/AP)", logical_id as u64, "entrando no scheduler");
    crate::sched::core::scheduler::run();
}
