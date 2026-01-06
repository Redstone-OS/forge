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
//!  │  └─────────────────────────────────┘ │
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

/// Endereço do trampoline (deve estar < 1MB para modo real)
/// Usamos 0x8000 que é uma área livre na conventional memory
pub const TRAMPOLINE_ADDR: u64 = 0x8000;

/// Timeout para esperar AP acordar (em iterações)
const AP_STARTUP_TIMEOUT: u32 = 100_000;

/// Flag global para APs sinalizarem que acordaram
/// Cada bit representa um AP (bit N = AP com logical_id N)
static AP_READY_FLAGS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Inicializa o subsistema de SMP bringup
pub fn init() {
    crate::kinfo!("(SMP/Bringup) Inicializado");
}

/// Acorda todos os APs
///
/// # Safety
///
/// - O trampoline code deve estar carregado em TRAMPOLINE_ADDR
/// - A topologia deve estar inicializada
pub unsafe fn bringup_all_aps() {
    let topo = topology::get();
    let cpu_count = topo.count();

    if cpu_count <= 1 {
        crate::kinfo!("(SMP/Bringup) Sistema single-core, pulando bringup");
        return;
    }

    crate::kinfo!("(SMP/Bringup) Acordando", (cpu_count - 1) as u64, "APs...");

    // Preparar trampoline (TODO: copiar código real para TRAMPOLINE_ADDR)
    // prepare_trampoline();

    // Acordar cada AP
    for cpu in topo.iter_aps() {
        match wake_ap(cpu.apic_id, cpu.logical_id) {
            Ok(()) => {
                crate::kdebug!("(SMP/Bringup) AP", cpu.logical_id as u64, "acordado");
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
    crate::kinfo!("(SMP/Bringup) ", ready_count as u64, "APs prontos");
}

/// Acorda um AP específico
///
/// # Argumentos
///
/// - `apic_id`: APIC ID de hardware do AP
/// - `logical_id`: ID lógico (0 a N-1)
unsafe fn wake_ap(apic_id: u32, logical_id: u32) -> Result<(), &'static str> {
    // 1. Enviar INIT IPI
    lapic::send_init_ipi(apic_id);

    // 2. Esperar 10ms
    delay_ms(10);

    // 3. Enviar primeiro SIPI
    let sipi_vector = (TRAMPOLINE_ADDR >> 12) as u8;
    lapic::send_sipi(apic_id, sipi_vector);

    // 4. Esperar 200µs
    delay_us(200);

    // 5. Verificar se acordou
    if !check_ap_ready(logical_id) {
        // 6. Enviar segundo SIPI (retry)
        lapic::send_sipi(apic_id, sipi_vector);
        delay_us(200);

        // 7. Esperar com timeout
        if !wait_for_ap(logical_id, AP_STARTUP_TIMEOUT) {
            return Err("AP não respondeu após SIPI");
        }
    }

    // Marcar como online na topologia
    topology::get_mut().set_online(logical_id as usize);

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
    // Aproximação: ~1000 loops por µs em hardware moderno
    for _ in 0..(ms * 1000) {
        delay_us(1);
    }
}

/// Delay aproximado em microsegundos (busy-wait)
fn delay_us(us: u32) {
    // Aproximação muito grosseira, mas funciona para bringup
    for _ in 0..(us * 100) {
        core::hint::spin_loop();
    }
}

// =============================================================================
// Trampoline (TODO)
// =============================================================================

/// Prepara o código trampoline para APs
///
/// O trampoline é código 16-bit que:
/// 1. Habilita A20
/// 2. Carrega GDT temporário
/// 3. Entra em protected mode
/// 4. Entra em long mode
/// 5. Salta para ap_entry() em Rust
///
/// TODO: Implementar quando tivermos assembly do trampoline
#[allow(dead_code)]
unsafe fn prepare_trampoline() {
    // Copiar código para TRAMPOLINE_ADDR
    // Configurar stack pointer
    // Configurar página de entrada
}

/// Ponto de entrada Rust para APs
///
/// Chamado pelo trampoline após entrar em long mode.
#[no_mangle]
pub extern "C" fn ap_entry(logical_id: u32) -> ! {
    // 1. Configurar GDT/IDT local
    // 2. Habilitar LAPIC
    unsafe {
        lapic::init();
    }

    // 3. Sinalizar que acordou
    ap_signal_ready(logical_id);

    // 4. Esperar scheduler inicializar
    loop {
        // TODO: Entrar no scheduler quando pronto
        crate::arch::x86_64::cpu::Cpu::halt();
    }
}
