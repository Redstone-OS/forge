/// Arquivo: core/smp/percpu.rs
///
/// Propósito: Gerenciamento de variáveis Por-CPU (Per-CPU variables).
/// Permite definir dados que possuem uma instância separada para cada núcleo do processador,
/// evitando contenda de locks (cache contention) e melhorando escalabilidade.
///
/// Detalhes de Implementação:
/// - Abordagem baseada em Array: `PerCpu<T>` mantém um array `[T; MAX_CPUS]`.
/// - O acesso é indexado pelo ID da CPU atual (`crate::arch::Cpu::current_core_id()`).
/// - Em x86_64, a arquitetura pode otimizar isso usando o segmento GS, mas aqui usamos
///   uma abstração genérica segura baseada no trait Cpu.

//! Variáveis Per-CPU

use core::cell::UnsafeCell;
use crate::arch::_traits::cpu::CpuTrait;

/// Número máximo de CPUs suportadas.
/// TODO: Tornar configurável via cfg
pub const MAX_CPUS: usize = 32;

/// Wrapper para dados que são replicados por CPU.
///
/// # Exemplo
///
/// ```ignore
/// static COUNTER: PerCpu<u64> = PerCpu::new(0);
/// 
/// fn inc() {
///     let val = COUNTER.get_mut();
///     *val += 1;
/// }
/// ```
pub struct PerCpu<T> {
    // UnsafeCell permite mutabilidade interior, necessário pois statics são imutáveis
    // e o acesso per-cpu é logicamente "thread-local" (mas requer cuidado com preempção/interrupção).
    data: [UnsafeCell<T>; MAX_CPUS],
}

// PerCpu é Sync se T for Send (pois cada CPU acessa o seu slot exclusivo).
// Na verdade, se garantirmos que apenas a CPU N acessa o slot N, nem precisamos de Sync no T,
// mas para inicialização e destruição talvez. Send é seguro.
unsafe impl<T: Send> Sync for PerCpu<T> {}

impl<T: Copy> PerCpu<T> {
    /// Cria uma nova variável PerCpu.
    /// Requer que T seja Copy para inicializar o array (const).
    pub const fn new(initial_value: T) -> Self {
        // Inicialização manual para 32 CPUs pois UnsafeCell não é Copy
        Self {
            data: [
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
                UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value), UnsafeCell::new(initial_value),
            ],
        }
    }

    /// Obtém uma referência mutável para o dado da CPU atual.
    ///
    /// # Safety
    ///
    /// - O chamador deve garantir que a preempção (troca de thread) ou migração de CPU
    ///   está desabilitada durante o uso da referência, ou que não importa se migrarmos.
    /// - Interrupções também devem ser consideradas se o dado for acessado em IRQ handlers.
    pub fn get(&self) -> &T {
        let cpu_id = crate::arch::x86_64::cpu::Cpu::current_core_id() as usize;
        
        if cpu_id >= MAX_CPUS {
            // Fallback seguro ou panic (idealmente panic, mas em kernel evitamos em caminhos críticos)
            // Retornamos o do core 0 em caso de erro catastrófico de topologia
            unsafe { &*self.data[0].get() }
        } else {
            unsafe { &*self.data[cpu_id].get() }
        }
    }

    /// Obtém uma referência mutável para o dado da CPU atual.
    ///
    /// # Safety
    ///
    /// O wrapper garante acesso exclusivo *por CPU*.
    /// - Deve ser chamado com preempção desabilitada para garantir atomicidade lógica
    ///   (para não trocarmos de tarefa no meio e outra tarefa na mesma CPU acessar).
    #[allow(clippy::mut_from_sync)]
    pub fn get_mut(&self) -> &mut T {
        let cpu_id = crate::arch::x86_64::cpu::Cpu::current_core_id() as usize;
        
        if cpu_id >= MAX_CPUS {
            unsafe { &mut *self.data[0].get() }
        } else {
            unsafe { &mut *self.data[cpu_id].get() }
        }
    }

    /// Acesso direto a uma CPU específica (útil para inicialização/debug)
    pub unsafe fn get_for_cpu(&self, cpu_id: usize) -> &mut T {
        if cpu_id >= MAX_CPUS {
            &mut *self.data[0].get()
        } else {
            &mut *self.data[cpu_id].get()
        }
    }
}
