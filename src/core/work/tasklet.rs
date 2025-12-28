/// Arquivo: core/work/tasklet.rs
///
/// Propósito: Implementação de Tasklets (Mini-tarefas atômicas).
/// Diferente de WorkQueues, Tasklets rodam em contexto atômico (SoftIRQ) e NÃO PODEM DORMIR.
/// São usados para processamento de interrupção de alta prioridade e baixa latência (ex: processamento de pacotes de rede).
///
/// Regras:
/// 1. Tasklets são serializados (mesmo tipo não roda em 2 CPUs ao mesmo tempo).
/// 2. Tasklets sempre rodam na CPU que os agendou (cache locality).
/// 3. Tasklets têm prioridade maior que WorkQueues e Threads normais.

//! Tasklets (execução atômica diferida)

use alloc::boxed::Box;
use core::sync::atomic::{AtomicU32, Ordering};

// Estados do Tasklet
const TASKLET_STATE_SCHED: u32 = 1 << 0; // Agendado para execução
const TASKLET_STATE_RUN:   u32 = 1 << 1; // Executando no momento

/// Estrutura de Tasklet
pub struct Tasklet {
    state: AtomicU32,
    func: Box<dyn FnMut() + Send + Sync>,
    data: u64, // Dado opcional de usuário
}

impl Tasklet {
    pub fn new<F>(func: F, data: u64) -> Self
    where
        F: FnMut() + Send + Sync + 'static,
    {
        Self {
            state: AtomicU32::new(0),
            func: Box::new(func),
            data,
        }
    }

    /// Agenda o tasklet para execução.
    /// Retorna `true` se agendou com sucesso, `false` se já estava agendado.
    pub fn schedule(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);
        
        // Se já está agendado, não faz nada
        if (state & TASKLET_STATE_SCHED) != 0 {
            return false;
        }

        // Tenta marcar como agendado atomicamente
        if self.state.fetch_or(TASKLET_STATE_SCHED, Ordering::Acquire) & TASKLET_STATE_SCHED != 0 {
            return false;
        }

        // TODO: Adicionar à lista de tasklets da CPU atual (PerCpu Vector)
        // TODO: Disparar SoftIRQ de Tasklet
        
        true
    }

    /// Executa o tasklet. Chamado internamente pelo SoftIRQ handler.
    ///
    /// # Safety
    ///
    /// Deve ser chamado em contexto atômico seguro (SoftIRQ).
    pub unsafe fn run(&mut self) {
        // Marca como rodando...
        // Nota: Em kernels reais temos spinlock para evitar reentrância em outros cores.
        // Aqui simplificamos assumindo que o handler de softirq garante serialização per-cpu.

        // Limpa bit SCHED e seta RUN
        let current = self.state.load(Ordering::Relaxed);
        // Se não estava agendado, erro de lógica, mas ignoramos
        if (current & TASKLET_STATE_SCHED) == 0 {
            return;
        }
        
        // Executa
        (self.func)();
        
        // Limpa estado (assumindo que RUN não é persistente entre chamadas na nossa impl simplificada)
        self.state.store(0, Ordering::Release);
    }
}
