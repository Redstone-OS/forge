/// Arquivo: core/power/cpuidle.rs
///
/// Propósito: Gerenciamento de Estados de Ociosidade da CPU (C-States).
/// Quando não há tarefas para rodar, o SO deve colocar a CPU em modos de baixo consumo.
///
/// Detalhes de Implementação:
/// - C0: Operando.
/// - C1: HLT (Halt).
/// - C2+: Modos mais profundos (que param clocks e caches), requerendo suporte de hardware (MWAIT/ACPI).

//! CPU Idle Management

/// Informações sobre um estado de idle (C-State)
pub struct IdleState {
    pub name: &'static str,
    pub latency_ns: u64, // Tempo para sair do estado
    pub power_usage: u32, // Consumo relativo
}

/// Chamado pelo loop "idle" do scheduler quando não há trabalho.
///
/// # Loop de Idle
/// 1. Desabilita interrupções (via Cpu::halt wrapper que geralmente é STI; HLT atomicamente ou CLI; check; HLT).
/// 2. Escolhe melhor estado C (Governance).
/// 3. Entra no estado.
pub fn enter_idle_loop() -> ! {
    loop {
        // TODO: Verificar se há callbacks de RCU ou SoftIRQs pendentes antes de dormir.
        
        // Caminho simples: C1 (HLT)
        // Isso coloca a CPU em pause até a próxima interrupção.
        
        unsafe {
            // Em x86: HLT para até a próxima interrupção.
            // As interrupções DEVEM estar habilitadas para acordar.
            // O trait CpuTrait::halt() geralmente faz isso.
            crate::arch::Cpu::halt();
        }
        
        // Ao acordar, verificamos se precisamos reagendar.
        // O scheduler cuidará disso se for uma interrupção de timer.
    }
}

/// Seleciona o melhor estado de idle baseado na previsão de tempo ocioso.
/// (Placeholder para futuro Governor)
pub fn select_idle_state() -> &'static IdleState {
    &C1_STATE
}

static C1_STATE: IdleState = IdleState {
    name: "C1 (HALT)",
    latency_ns: 0,
    power_usage: 100,
};
