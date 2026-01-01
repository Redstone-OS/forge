//! # Gerenciamento de CPU e Localidade
//!
//! Este arquivo define a abstração para uma unidade de processamento (CPU Lógica).
//! Em sistemas modernos, o agendador deve ser "consciente" de qual CPU está rodando,
//! permitindo que cada núcleo gerencie suas próprias filas e interrupções,
//! minimizando a contenção de locks globais.
//!
//! ## Responsabilidades Futuras (SMP & Modern OS):
//! - **Local APIC (LAPIC):** Cada CPU precisa gerenciar seu próprio controlador de interrupções.
//! - **TSS (Task State Segment):** Necessário para que cada CPU tenha sua própria stack de interrupção segura.
//! - **Per-CPU RunQueues:** Para escala massiva, cada CPU deve ter sua própria fila de tarefas.
//! - **Topologia:** Conhecer a hierarquia (Núcleo -> Thread -> Pacote/Socket) para otimizar cache.
//! - **IPI (Inter-Processor Interrupts):** Mecanismo para uma CPU "acordar" a outra.

use crate::sys::types::Tid;

/// Estrutura que representa uma CPU lógica e seu estado privado.
pub struct Cpu {
    /// ID da CPU (ID de hardware, geralmente o APIC ID inicial)
    pub id: u32,

    /// ID da tarefa que está ocupando este núcleo no momento.
    pub current_tid: Option<Tid>,

    /// Flag de preempção: sinaliza que a tarefa atual deve ser substituída.
    /// TODO: Migrar para AtomicBool para permitir check sem lock.
    pub need_resched: bool,
    // --- PLACEHOLDERS PARA O FUTURO ---

    // /// TODO: Fila de execução privada desta CPU (Evita lock na RUNQUEUE global)
    // pub runqueue: Option<PerCpuRunQueue>,

    // /// TODO: Ponteiro para o TSS desta CPU (Necessário para Ring 3 -> Ring 0 fixo)
    // pub tss_ptr: *mut TSS,

    // /// TODO: Estatísticas de uso (Idle time, System time, User time) por núcleo
    // pub stats: CpuStats,

    // /// TODO: Info de topologia (Para agendamento consciente de cache/NUMA)
    // pub topology: CpuTopology,
}

impl Cpu {
    /// Cria uma nova estrutura de CPU (Chamado durante o boot para cada núcleo detectado).
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            current_tid: None,
            need_resched: false,
        }
    }
}

/// Instância da CPU atual (Single Core por enquanto).
/// Em SMP, isso será um array ou mapeamento via GS-base.
pub static LOCAL_CPU: crate::sync::Spinlock<Cpu> = crate::sync::Spinlock::new(Cpu::new(0));

/// Sinaliza que esta CPU precisa realizar um re-agendamento.
#[no_mangle]
pub extern "C" fn set_need_resched() {
    LOCAL_CPU.lock().need_resched = true;
}

/// Verifica se esta CPU precisa realizar um re-agendamento.
#[no_mangle]
pub extern "C" fn should_reschedule() -> bool {
    LOCAL_CPU.lock().need_resched
}

/// Limpa a flag de re-agendamento desta CPU.
#[no_mangle]
pub extern "C" fn clear_need_resched() {
    LOCAL_CPU.lock().need_resched = false;
}

/// Balanceador de Carga (Load Balancer)
pub struct LoadBalancer;

impl LoadBalancer {
    /// Verifica se é necessário balancear carga entre CPUs.
    ///
    /// Em sistemas SMP, isso moveria tarefas de uma runqueue cheia para uma vazia.
    /// Atualmente (Single Core), é um no-op.
    pub fn balance() {
        // Futuro: Verificar disparidade entre runqueues[]
        // Se disparidade > threshold, migrar tasks.
    }

    /// Retorna a carga estimada do sistema (ex: número de tasks ready).
    pub fn get_load() -> u64 {
        // Retorna tamanho da runqueue global por enquanto
        super::runqueue::RUNQUEUE.lock().len() as u64
    }
}
