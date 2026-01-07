//! # Per-CPU Data Structures
//!
//! Estruturas de dados locais por CPU para o scheduler multi-core.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                      PER-CPU DATA                       │
//! ├─────────────────────────────────────────────────────────┤
//! │                                                         │
//! │   CPUS[0]            CPUS[1]           CPUS[2]          │
//! │   ┌──────────┐      ┌──────────┐      ┌──────────┐      │
//! │   │ cpu_id   │      │ cpu_id   │      │ cpu_id   │      │
//! │   │ state    │      │ state    │      │ state    │      │
//! │   │ current  │      │ current  │      │ current  │      │
//! │   │ runqueue │      │ runqueue │      │ runqueue │      │
//! │   │ idle_task│      │ idle_task│      │ idle_task│      │
//! │   │ resched  │      │ resched  │      │ resched  │      │
//! │   └──────────┘      └──────────┘      └──────────┘      │
//! │                                                         │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Acesso
//!
//! - `this_cpu_id()` → ID da CPU atual (via LAPIC, lock-free)
//! - `this_cpu()` → Dados da CPU atual (retorna guard do lock)
//! - `cpu(id)` → Dados de uma CPU específica
//!
//! ## Invariantes
//!
//! - Cada CPU só modifica seus próprios dados durante `schedule()`
//! - `balance()` é a única função que acessa dados de outras CPUs
//! - Lock order: sempre menor CPU ID primeiro para evitar deadlock

use crate::sched::Task;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};

/// Número máximo de CPUs suportadas
pub const MAX_CPUS: usize = 64;

/// Estado de uma CPU
///
/// ```text
/// [Offline] ──SIPI──► [Booting] ──init──► [Idle] ◄───► [Running]
///                                             │             │
///                                             └──►[Panic]◄──┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuState {
    /// CPU não inicializada ou desligada
    Offline = 0,
    /// CPU recebeu SIPI, executando trampoline
    Booting = 1,
    /// CPU inicializada, executando idle task
    Idle = 2,
    /// CPU executando task normal
    Running = 3,
    /// CPU em estado de pânico (erro fatal)
    Panic = 4,
}

/// Dados locais de uma CPU
///
/// Cada CPU tem sua própria instância, protegida por Spinlock.
/// O Spinlock já desabilita interrupções automaticamente.
pub struct PerCpuData {
    /// ID lógico desta CPU (0, 1, 2, ...)
    pub cpu_id: usize,

    /// Estado atual da CPU
    pub state: CpuState,

    /// Task atualmente em execução nesta CPU
    pub current: Option<Pin<Box<Task>>>,

    /// Fila local de tasks prontas para esta CPU
    pub runqueue: VecDeque<Pin<Box<Task>>>,

    /// Idle task permanente desta CPU
    /// Nunca é removida, nunca entra na runqueue
    pub idle_task: Option<Pin<Box<Task>>>,

    /// Flag indicando que preempção foi solicitada
    /// Verificada no retorno de interrupção
    pub need_resched: AtomicBool,
}

impl PerCpuData {
    /// Cria estrutura vazia (para inicialização tardia)
    pub const fn empty() -> Self {
        Self {
            cpu_id: 0,
            state: CpuState::Offline,
            current: None,
            runqueue: VecDeque::new(),
            idle_task: None,
            need_resched: AtomicBool::new(false),
        }
    }

    /// Inicializa com ID específico
    pub fn init(&mut self, cpu_id: usize) {
        self.cpu_id = cpu_id;
        self.state = CpuState::Booting;
        self.current = None;
        self.runqueue = VecDeque::new();
        self.idle_task = None;
        self.need_resched = AtomicBool::new(false);
    }

    /// Solicita reescalonamento
    #[inline]
    pub fn request_resched(&self) {
        self.need_resched.store(true, Ordering::Release);
    }

    /// Verifica e limpa flag de reescalonamento
    #[inline]
    pub fn should_resched(&self) -> bool {
        self.need_resched.swap(false, Ordering::AcqRel)
    }

    /// Adiciona task à runqueue local
    pub fn enqueue(&mut self, task: Pin<Box<Task>>) {
        self.runqueue.push_back(task);
    }

    /// Remove próxima task da runqueue
    pub fn dequeue(&mut self) -> Option<Pin<Box<Task>>> {
        self.runqueue.pop_front()
    }

    /// Número de tasks na runqueue
    pub fn runqueue_len(&self) -> usize {
        self.runqueue.len()
    }

    /// Verifica se runqueue está vazia
    pub fn runqueue_is_empty(&self) -> bool {
        self.runqueue.is_empty()
    }
}

// =============================================================================
// ARRAY GLOBAL DE CPUS
// =============================================================================

/// Array global de estruturas per-CPU
///
/// Cada elemento é um Option dentro de Spinlock.
/// - `None` = CPU não inicializada
/// - `Some(data)` = CPU inicializada
///
/// O Spinlock já desabilita interrupções, garantindo segurança.
pub static CPUS: [Spinlock<Option<PerCpuData>>; MAX_CPUS] = {
    // const-init array de Spinlocks
    const INIT: Spinlock<Option<PerCpuData>> = Spinlock::new(None);
    [INIT; MAX_CPUS]
};

/// Flag indicando se o sistema per-CPU foi inicializado
static PER_CPU_INITIALIZED: AtomicBool = AtomicBool::new(false);

// =============================================================================
// FUNÇÕES DE ACESSO
// =============================================================================

/// Retorna o ID da CPU atual
///
/// # Importante
///
/// Esta função é **lock-free** e usa o LAPIC ID diretamente.
/// Funciona mesmo antes do scheduler estar inicializado.
#[inline]
pub fn this_cpu_id() -> usize {
    // Usa LAPIC ID diretamente - nunca falha
    crate::arch::x86_64::apic::lapic::current_apic_id() as usize
}

/// Inicializa dados da CPU especificada
///
/// Deve ser chamado uma vez durante o boot de cada CPU,
/// antes de entrar no scheduler.
pub fn init_cpu(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        crate::kerror!("(PerCpu) CPU ID", cpu_id as u64, "excede MAX_CPUS!");
        return;
    }

    let mut guard = CPUS[cpu_id].lock();
    if guard.is_some() {
        crate::kwarn!("(PerCpu) CPU", cpu_id as u64, "já inicializada!");
        return;
    }

    let mut data = PerCpuData::empty();
    data.init(cpu_id);
    *guard = Some(data);

    PER_CPU_INITIALIZED.store(true, Ordering::Release);
    crate::kdebug!("(PerCpu) CPU", cpu_id as u64, "inicializada");
}

/// Verifica se a CPU foi inicializada
#[inline]
pub fn is_cpu_initialized(cpu_id: usize) -> bool {
    if cpu_id >= MAX_CPUS {
        return false;
    }
    CPUS[cpu_id].lock().is_some()
}

/// Verifica se o sistema per-CPU foi inicializado
#[inline]
pub fn is_initialized() -> bool {
    PER_CPU_INITIALIZED.load(Ordering::Acquire)
}

/// Solicita reescalonamento na CPU atual
#[inline]
pub fn set_need_resched() {
    let cpu_id = this_cpu_id();
    if let Some(ref data) = *CPUS[cpu_id].lock() {
        data.need_resched.store(true, Ordering::Release);
    }
}

/// Verifica se reescalonamento é necessário na CPU atual
#[inline]
pub fn should_resched() -> bool {
    let cpu_id = this_cpu_id();
    if let Some(ref data) = *CPUS[cpu_id].lock() {
        data.need_resched.swap(false, Ordering::AcqRel)
    } else {
        false
    }
}

// TODO: Verificar se é necessário remover
// =============================================================================
// FUNÇÕES EXTERNAS (#[no_mangle])
// =============================================================================

/// Verifica se deve reescalonar (chamada por código externo/assembly)
#[no_mangle]
pub extern "C" fn should_reschedule() -> bool {
    let cpu_id = this_cpu_id();
    if let Some(ref data) = *CPUS[cpu_id].lock() {
        data.need_resched.load(Ordering::Acquire)
    } else {
        false
    }
}

/// Limpa flag de reescalonamento (chamada por código externo/assembly)
#[no_mangle]
pub extern "C" fn clear_need_resched() {
    let cpu_id = this_cpu_id();
    if let Some(ref data) = *CPUS[cpu_id].lock() {
        data.need_resched.store(false, Ordering::Release);
    }
}

// =============================================================================
// LOAD BALANCING (STUB)
// =============================================================================

/// Load Balancer para distribuir trabalho entre CPUs
pub struct LoadBalancer;

impl LoadBalancer {
    /// Verifica se é necessário balancear carga
    ///
    /// TODO: Implementar na Fase 3
    pub fn balance() {
        // Stub - será implementado depois
    }

    /// Retorna a carga total do sistema
    pub fn get_total_load() -> usize {
        let mut total = 0;
        for i in 0..MAX_CPUS {
            if let Some(ref data) = *CPUS[i].lock() {
                total += data.runqueue_len();
            }
        }
        total
    }

    /// Encontra CPU com menor carga
    pub fn find_idlest_cpu() -> Option<usize> {
        let mut min_load = usize::MAX;
        let mut min_cpu = None;

        for i in 0..MAX_CPUS {
            if let Some(ref data) = *CPUS[i].lock() {
                if data.state != CpuState::Offline && data.state != CpuState::Panic {
                    let load = data.runqueue_len();
                    if load < min_load {
                        min_load = load;
                        min_cpu = Some(i);
                    }
                }
            }
        }

        min_cpu
    }
}
