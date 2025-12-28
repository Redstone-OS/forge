use crate::sync::spinlock::Spinlock;
/// Arquivo: core/smp/topology.rs
///
/// Propósito: Gerenciar a topologia de processadores do sistema.
/// Mantém o registro de todos os CPUs detectados (via ACPI MADT ou Device Tree),
/// seus IDs (APIC ID, ACPI ID) e status (Online/Offline).
///
/// Detalhes de Implementação:
/// - Estrutura global `CPU_TOPOLOGY` que é populada durante o boot.
/// - Fundamental para o scheduler saber quantos cores existem.
// Topologia de CPUs (SMP)
use alloc::vec::Vec; // Assume que teremos spinlock

/// Identificador lógico de CPU (0 a N-1)
pub type CpuId = u32;

/// Informações sobre uma CPU detectada
#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    /// ID lógico atribuído pelo kernel (índice no vetor)
    pub logical_id: CpuId,

    /// ID de Hardware (APIC ID em x86, Hart ID em RISC-V)
    pub hw_id: u32,

    /// ID do Processador na ACPI
    pub acpi_id: u32,

    /// Indica se é o Bootstrap Processor (BSP)
    pub is_bsp: bool,

    /// Indica se a CPU está online e rodando
    pub online: bool,
}

pub struct CpuTopology {
    cpus: Vec<CpuInfo>,
    bsp_id: Option<CpuId>,
}

impl CpuTopology {
    pub const fn new() -> Self {
        Self {
            cpus: Vec::new(),
            bsp_id: None,
        }
    }

    /// Registra uma nova CPU descoberta
    pub fn register_cpu(&mut self, hw_id: u32, acpi_id: u32, is_bsp: bool) -> CpuId {
        let logical_id = self.cpus.len() as u32;

        let info = CpuInfo {
            logical_id,
            hw_id,
            acpi_id,
            is_bsp,
            online: is_bsp, // BSP já começa online
        };

        if is_bsp {
            self.bsp_id = Some(logical_id);
        }

        self.cpus.push(info);
        logical_id
    }

    /// Retorna o número total de CPUs detectadas
    pub fn count(&self) -> usize {
        self.cpus.len()
    }

    /// Itera sobre as CPUs
    pub fn iter(&self) -> core::slice::Iter<'_, CpuInfo> {
        self.cpus.iter()
    }
}

// Topologia global protegida por lock
// Nota: Vec exige alocação, então isso só pode ser usado após init do Heap.
// Antes do Heap, talvez precisemos de um array estático fixo ou usar memblock.
// Para simplificar, assumimos que a topologia é construída após o Heap ou usamos
// uma estrutura pré-alocada limitada se necessário.
pub static TOPOLOGY: Spinlock<CpuTopology> = Spinlock::new(CpuTopology::new());
