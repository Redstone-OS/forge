//! # Timer Driver - 100% Assembly Implementation
//!
//! Driver do PIT (Programmable Interval Timer) - Intel 8253/8254.
//!
//! ## GARANTIAS
//! - **Zero SSE/AVX**: Todo código I/O é assembly inline puro
//! - **Determinístico**: Comportamento idêntico em todos os perfis de compilação
//!
//! ## Responsabilidades
//! 1. Gerar o "Heartbeat" do sistema (Timer Interrupt)
//! 2. Contabilizar o tempo global (Ticks/Uptime)
//! 3. Acionar o Scheduler para preempção
//!
//! ## Limitações
//! - Frequência base fixa de ~1.19 MHz
//! - Depende do PIC (IRQ 0 → Vector 32)
//! - Não é preciso para medições de alta resolução (usar TSC/HPET para isso)

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// Portas de I/O do PIT
const PIT_CHANNEL0: u16 = 0x40; // Channel 0 data (System Timer)
const PIT_CHANNEL2: u16 = 0x42; // Channel 2 data (PC Speaker)
const PIT_COMMAND: u16 = 0x43; // Command register

// Frequência base do oscilador (~1.193182 MHz)
const PIT_BASE_FREQ: u32 = 1_193_182;

// Contador global de ticks (monotonic clock)
pub static TICKS: AtomicU64 = AtomicU64::new(0);

// Frequência atual configurada
static FREQUENCY: AtomicU32 = AtomicU32::new(0);

/// Inicializa o PIT com a frequência especificada.
///
/// # Arguments
/// * `freq_hz` - Frequência desejada em Hz (ex: 100 = 10ms tick)
///
/// # Returns
/// Frequência real configurada (pode diferir devido à precisão do divisor).
///
/// 100% Assembly I/O - Zero SSE.
#[inline(never)]
pub fn init(freq_hz: u32) -> u32 {
    if freq_hz == 0 || freq_hz > PIT_BASE_FREQ {
        return 0;
    }

    let divisor = PIT_BASE_FREQ / freq_hz;

    // Divisor máximo é 65535 (0 = 65536)
    let divisor = if divisor > 65535 {
        65535
    } else {
        divisor as u16
    };

    let actual_freq = PIT_BASE_FREQ / (divisor as u32);
    FREQUENCY.store(actual_freq, Ordering::Relaxed);

    unsafe {
        core::arch::asm!(
            // Command: Channel 0, Access mode lobyte/hibyte, Mode 2 (rate generator)
            // 0x34 = 00 11 010 0 = Channel 0, lo/hi, Mode 2, Binary
            // Ou usar Mode 3 (square wave) = 0x36
            "mov dx, {cmd}",
            "mov al, 0x36",      // Mode 3: Square Wave Generator
            "out dx, al",

            // Send divisor low byte
            "mov dx, {ch0}",
            "mov al, {div_lo}",
            "out dx, al",

            // Send divisor high byte
            "mov al, {div_hi}",
            "out dx, al",

            cmd = const PIT_COMMAND,
            ch0 = const PIT_CHANNEL0,
            div_lo = in(reg_byte) (divisor & 0xFF) as u8,
            div_hi = in(reg_byte) ((divisor >> 8) & 0xFF) as u8,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }

    actual_freq
}

/// Lê o contador atual do Channel 0.
///
/// Útil para medições precisas de tempo ou one-shot timing.
///
/// # Returns
/// Valor atual do contador (16-bit, decrementa de divisor até 0).
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn read_count() -> u16 {
    let count: u16;

    unsafe {
        core::arch::asm!(
            // Latch command for Channel 0 (0x00 = Channel 0, latch)
            "mov dx, {cmd}",
            "xor al, al",
            "out dx, al",

            // Read low byte
            "mov dx, {ch0}",
            "in al, dx",
            "mov ah, al",

            // Read high byte
            "in al, dx",
            "xchg al, ah",       // al = low, ah = high

            cmd = const PIT_COMMAND,
            ch0 = const PIT_CHANNEL0,
            out("ax") count,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }

    count
}

/// Handler de Interrupção do Timer (IRQ 0 / Vector 32).
///
/// Chamado pelo stub assembly em interrupts.rs.
/// Este é o "maestro" que dita o ritmo do sistema operacional.
///
/// # Responsabilidades
/// 1. **Timekeeping**: Incrementar contador de ticks (atômico)
/// 2. **Scheduling**: Verificar quantum e decidir context switch
/// 3. **Hardware ACK**: Enviar EOI ao PIC
/// 4. **Context Switch**: Executar troca de contexto se necessário
pub fn handle_interrupt() {
    // 1. Timekeeping (Crítico e Atômico)
    TICKS.fetch_add(1, Ordering::Relaxed);

    // 2. Scheduling
    // Verifica se a tarefa atual estourou seu quantum e precisa ser trocada
    let switch_info = {
        let mut sched = crate::sched::scheduler::SCHEDULER.lock();
        sched.schedule()
    };

    // 3. Hardware ACK
    // Enviar EOI para o PIC para podermos receber a próxima interrupção
    crate::drivers::pic::send_eoi(0); // IRQ 0

    // 4. Context Switch
    // Se o scheduler decidiu trocar de tarefa, realizar a troca agora.
    // DEVE ser a última coisa na função!
    if let Some((old_ptr, new_ptr)) = switch_info {
        unsafe {
            // Configurar TSS.rsp0 com a kstack da nova tarefa
            crate::arch::x86_64::gdt::set_kernel_stack(new_ptr);

            // Executar context switch
            crate::sched::context_switch(old_ptr as *mut u64, new_ptr);
        }
    }
}

/// Retorna o número total de ticks desde o boot.
#[inline(always)]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Retorna a frequência configurada atual em Hz.
#[inline(always)]
pub fn frequency() -> u32 {
    FREQUENCY.load(Ordering::Relaxed)
}

/// Retorna o tempo de uptime em segundos (aproximado).
///
/// A precisão depende da frequência configurada.
#[inline(always)]
pub fn uptime_seconds() -> u64 {
    let freq = frequency();
    if freq == 0 {
        return 0;
    }
    ticks() / (freq as u64)
}

/// Retorna o tempo de uptime em milissegundos (aproximado).
#[inline(always)]
pub fn uptime_ms() -> u64 {
    let freq = frequency();
    if freq == 0 {
        return 0;
    }
    (ticks() * 1000) / (freq as u64)
}

/// Espera ativa por um número de ticks.
///
/// **AVISO**: Bloqueante! Use apenas para delays curtos em early boot.
///
/// # Arguments
/// * `ticks_to_wait` - Número de ticks para esperar
pub fn delay_ticks(ticks_to_wait: u64) {
    let start = ticks();
    while ticks() - start < ticks_to_wait {
        core::hint::spin_loop();
    }
}

/// Espera ativa por um número de milissegundos.
///
/// **AVISO**: Bloqueante! Use apenas para delays curtos em early boot.
///
/// # Arguments
/// * `ms` - Tempo em milissegundos para esperar
pub fn delay_ms(ms: u64) {
    let freq = frequency();
    if freq == 0 {
        return;
    }
    let ticks_needed = (ms * freq as u64) / 1000;
    delay_ticks(ticks_needed);
}

// =============================================================================
// PC Speaker (Channel 2) - Opcional, para beeps de diagnóstico
// =============================================================================

/// Gera um beep no PC Speaker com a frequência especificada.
///
/// Usa Channel 2 do PIT conectado ao speaker via port 0x61.
///
/// # Arguments
/// * `freq_hz` - Frequência do tom em Hz (ex: 440 = Lá4)
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn beep_start(freq_hz: u32) {
    if freq_hz == 0 || freq_hz > PIT_BASE_FREQ {
        return;
    }

    let divisor = (PIT_BASE_FREQ / freq_hz) as u16;

    unsafe {
        core::arch::asm!(
            // Configure Channel 2 for square wave
            "mov dx, {cmd}",
            "mov al, 0xB6",      // Channel 2, lo/hi, Mode 3
            "out dx, al",

            // Set frequency divisor
            "mov dx, {ch2}",
            "mov al, {div_lo}",
            "out dx, al",
            "mov al, {div_hi}",
            "out dx, al",

            // Enable speaker (bits 0 and 1 of port 0x61)
            "mov dx, 0x61",
            "in al, dx",
            "or al, 0x03",
            "out dx, al",

            cmd = const PIT_COMMAND,
            ch2 = const PIT_CHANNEL2,
            div_lo = in(reg_byte) (divisor & 0xFF) as u8,
            div_hi = in(reg_byte) ((divisor >> 8) & 0xFF) as u8,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Para o beep do PC Speaker.
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn beep_stop() {
    unsafe {
        core::arch::asm!(
            // Disable speaker (clear bits 0 and 1 of port 0x61)
            "mov dx, 0x61",
            "in al, dx",
            "and al, 0xFC",
            "out dx, al",

            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Toca um beep curto para diagnóstico.
///
/// Útil para indicar checkpoints durante boot sem depender de vídeo/serial.
///
/// # Arguments
/// * `freq_hz` - Frequência do beep
/// * `duration_ms` - Duração em milissegundos
pub fn beep(freq_hz: u32, duration_ms: u64) {
    beep_start(freq_hz);
    delay_ms(duration_ms);
    beep_stop();
}
