//! Lógica de Idle (Ociosidade)

use super::scheduler::{pick_next, CURRENT};
use super::switch::prepare_and_switch_to;
use crate::arch::Cpu;
use crate::sched::task::context::CpuContext;

/// Entra no loop de idle aguardando tasks ficarem prontas.
///
/// # Safety
/// Deve ser chamado com interrupções desabilitadas e lock do scheduler liberado.
pub unsafe fn enter_idle_loop(old_ctx: Option<*mut CpuContext>) -> ! {
    crate::kinfo!("(Sched) Nenhuma task disponível. Entrando em modo Idle.");

    let mut idle_count = 0u64;
    loop {
        // Habilita interrupções e espera o próximo evento de hardware
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();

        idle_count += 1;
        if idle_count % 100 == 0 {
            crate::ktrace!("(Sched) Idler pulsação:", idle_count);
        }

        // Verifica se há alguma task na runqueue agora
        if let Some(next) = pick_next() {
            crate::kinfo!("(Sched) Task acordou! Retomando escalonamento.");

            // Re-adquire o lock necessário para o switch
            let g = CURRENT.lock();
            prepare_and_switch_to(next, old_ctx, g);
            // prepare_and_switch_to não retorna
        }
    }
}

/// Loop de idle contínuo quando o sistema está totalmente vazio.
pub unsafe fn system_idle_loop() -> ! {
    crate::kinfo!("(Sched) Sistema ocioso. Aguardando interrupções...");
    loop {
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();

        if let Some(next) = pick_next() {
            let g = CURRENT.lock();
            prepare_and_switch_to(next, None, g);
        }
    }
}
