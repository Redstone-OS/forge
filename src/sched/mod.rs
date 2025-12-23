//! Módulo de Agendamento (Scheduler).

pub mod context;
pub mod scheduler;
pub mod task;

// Re-exportar assembly switch
core::arch::global_asm!(include_str!("../arch/x86_64/switch.s"));

extern "C" {
    /// Troca de contexto em assembly.
    ///
    /// # Arguments
    /// * `old_stack_ptr`: Endereço onde salvar o RSP atual (ou 0 se não salvar).
    /// * `new_stack_ptr`: Valor do novo RSP a carregar.
    pub fn context_switch(old_stack_ptr: u64, new_stack_ptr: u64);
}
