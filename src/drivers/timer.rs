//! Driver do PIT (Programmable Interval Timer) - Intel 8253/8254.
//!
//! Este é o timer legado da arquitetura x86. Em sistemas modernos, ele atua como
//! fallback ou timer de boot até que o APIC Timer ou HPET sejam inicializados.
//!
//! # Responsabilidades
//! 1. Gerar o "Heartbeat" do sistema (Timer Interrupt).
//! 2. Contabilizar o tempo global (Ticks/Uptime).
//! 3. Acionar o Scheduler para preempção.
//!
//! # Limitações
//! - Frequência base fixa de ~1.19 MHz.
//! - Depende do PIC (IRQ 0) ou IO-APIC (IRQ 2 override).
//! - Não é preciso para medições de alta resolução (usar TSC/HPET para isso).

use crate::arch::x86_64::ports::Port;
use crate::sync::Mutex;
use crate::sys::Errno;
use core::sync::atomic::{AtomicU64, Ordering};

/// Frequência base do oscilador do PIT (1.193182 MHz).
const BASE_FREQUENCY: u32 = 1_193_182;

// Portas de I/O do PIT
const PORT_CHANNEL0: u16 = 0x40; // Canal 0 (System Timer)
const PORT_COMMAND: u16 = 0x43; // Registrador de Comando

/// Contador global de ticks do sistema (Monotonic Clock).
/// Incrementado a cada interrupção do timer.
pub static TICKS: AtomicU64 = AtomicU64::new(0);

/// Driver do Programmable Interval Timer.
pub struct Pit {
    channel0: Port<u8>,
    command: Port<u8>,
    frequency: u32,
}

impl Pit {
    /// Cria uma interface para o PIT.
    ///
    /// # Safety
    /// O caller deve garantir que está rodando em hardware compatível com x86/IBM PC
    /// e que tem acesso exclusivo às portas 0x40 e 0x43.
    pub const unsafe fn new() -> Self {
        Self {
            channel0: Port::new(PORT_CHANNEL0),
            command: Port::new(PORT_COMMAND),
            frequency: 0,
        }
    }

    /// Configura a frequência do Timer (em Hz).
    ///
    /// # Arguments
    /// * `freq`: Frequência desejada (ex: 100Hz = 10ms).
    ///
    /// # Returns
    /// * `Ok(u32)`: Frequência real configurada (devido à precisão do divisor).
    /// * `Err(Errno)`: Se a frequência for inválida (0 ou muito alta).
    pub fn set_frequency(&mut self, freq: u32) -> Result<u32, Errno> {
        crate::kdebug!("(PIT) set_frequency: Desejado {} Hz", freq);

        if freq == 0 || freq > BASE_FREQUENCY {
            crate::kwarn!("(PIT) set_frequency: Frequência inválida");
            return Err(Errno::EINVAL);
        }

        // Divisor = Base / Freq
        let divisor = BASE_FREQUENCY / freq;

        // O divisor deve caber em 16 bits (exceto 0 que significa 65536)
        if divisor > 65535 {
            crate::kwarn!("(PIT) set_frequency: Divisor muito grande para {} Hz", freq);
            return Err(Errno::EINVAL); // Frequência muito baixa (< 18.2 Hz)
        }

        let actual_freq = BASE_FREQUENCY / divisor;
        crate::ktrace!(
            "(PIT) set_frequency: Divisor calculado: {} (Freq real: {} Hz)",
            divisor,
            actual_freq
        );

        unsafe {
            // Modo de Operação:
            self.command.write(0x36);
            crate::ktrace!("(PIT) set_frequency: Comando 0x36 (Square Wave) enviado");

            // Enviar divisor (Low byte, depois High byte)
            self.channel0.write((divisor & 0xFF) as u8);
            self.channel0.write((divisor >> 8) as u8);
            crate::ktrace!("(PIT) set_frequency: LSB/MSB do divisor enviados");
        }

        self.frequency = actual_freq;
        crate::kinfo!("(PIT) Frequência configurada para {} Hz", actual_freq);
        Ok(actual_freq)
    }

    /// Retorna a frequência atual configurada.
    pub fn frequency(&self) -> u32 {
        self.frequency
    }
}

/// Instância global do PIT protegida por Mutex.
/// Usada apenas para configuração inicial; o handler de interrupção não precisa lockar isso
/// para incrementar ticks (usa Atomic).
pub static PIT: Mutex<Pit> = Mutex::new(unsafe { Pit::new() });

/// Handler de Interrupção do Timer (IRQ 0 / Vector 32).
///
/// Chamado pelo stub assembly (`interrupts.rs`).
/// Este é o "maestro" que dita o ritmo do sistema operacional.
pub fn handle_timer_interrupt() {
    // 1. Timekeeping (Crítico e Atômico)
    TICKS.fetch_add(1, Ordering::Relaxed);

    // 2. Scheduling (Política)
    // Verifica se a tarefa atual estourou seu quantum e precisa ser trocada.
    // O lock do scheduler deve ser rápido. Em sistemas RT, isso seria separado.
    let switch_info = {
        // Tenta lockar o scheduler. Se falhar (reentrância?), pulamos este tick.
        // Em um kernel simples, lock direto é aceitável se garantirmos que IRQs estão off no lock.
        let mut sched = crate::sched::scheduler::SCHEDULER.lock();
        sched.schedule()
    };

    // 3. Hardware ACK (Mecanismo)
    // Avisa o PIC que a interrupção foi processada para recebermos a próxima.
    // TODO: Abstrair via trait `InterruptController` para suportar APIC futuramente.
    unsafe {
        crate::drivers::pic::PICS.lock().notify_eoi(32); // 32 = Vetor remapeado do IRQ0
    }

    // 4. Context Switch (Despacho)
    // Se o scheduler decidiu trocar de tarefa, realizamos a troca de pilhas agora.
    // Isso deve ser a ÚLTIMA coisa feita na função, pois não retornaremos para a linha seguinte
    // no contexto da tarefa antiga imediatamente.
    if let Some((old_ptr, new_ptr)) = switch_info {
        unsafe {
            // IMPORTANTE: Configurar TSS.rsp0 com a kstack da nova tarefa
            // Quando uma interrupção ocorre em Ring 3, a CPU usa TSS.rsp0 como kernel stack
            crate::arch::x86_64::gdt::set_kernel_stack(new_ptr);

            crate::sched::context_switch(old_ptr as *mut u64, new_ptr);
        }
    }
}

/// Retorna o tempo de atividade do sistema em segundos (aproximado).
pub fn uptime_seconds() -> u64 {
    let ticks = TICKS.load(Ordering::Relaxed);
    // Assumindo 100Hz (configurado no entry.rs).
    // TODO: Ler a frequência real do driver em vez de hardcoded.
    ticks / 100
}
