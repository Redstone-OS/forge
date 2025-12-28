//! Arquivo: core/power/cpufreq.rs
//!
//! Propósito: Escalonamento de Frequência da CPU (DVFS).
//! Permite ajustar dinamicamente a frequência e voltagem da CPU para
//! economizar energia ou maximizar performance.
//!
//! Detalhes de Implementação:
//! - Abstração de P-States (ACPI).
//! - Suporte a Governadores (Performance, Powersave, Ondemand).

//! CPU Frequency Scaling

/// Unidade de frequência em KHz
pub type FrequencyKHz = u32;

/// Política de frequência para uma CPU (ou grupo de CPUs)
pub struct CpuFreqPolicy {
    pub min_freq: FrequencyKHz,
    pub max_freq: FrequencyKHz,
    pub current_freq: FrequencyKHz,
    pub governor: &'static str, // Nome do governador ativo
}

/// Interface para o driver de hardware (ex: intel_pstate, acpi-cpufreq)
pub trait CpuFreqDriver: Send + Sync {
    /// Inicializa o driver para a CPU especificada
    fn init(&self, cpu_id: u32) -> Result<(), &'static str>;

    /// Define a frequência alvo
    fn set_target(&self, cpu_id: u32, freq: FrequencyKHz) -> Result<(), &'static str>;

    /// Obtém a frequência atual
    fn get(&self, cpu_id: u32) -> FrequencyKHz;
}

/// Interface para algoritmos de decisão (Governors)
pub trait Governor {
    fn name(&self) -> &'static str;

    /// Chamado periodicamente ou em eventos de carga para decidir a nova frequência
    fn update(&self, policy: &mut CpuFreqPolicy, load: u32) -> FrequencyKHz;
}

// Implementação dummy do Governor "Performance"
pub struct PerformanceGovernor;

impl Governor for PerformanceGovernor {
    fn name(&self) -> &'static str {
        "performance"
    }

    fn update(&self, policy: &mut CpuFreqPolicy, _load: u32) -> FrequencyKHz {
        // Sempre máximo
        policy.max_freq
    }
}
