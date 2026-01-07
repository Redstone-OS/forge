//! # Idle Task - Tarefa Ociosa Per-CPU
//!
//! Cada CPU tem sua própria idle task que nunca é removida.
//!
//! ## Arquitetura
//!
//! ```text
//!   CPU 0             CPU 1             CPU 2
//!   ┌─────────┐      ┌─────────┐      ┌─────────┐
//!   │ idle/0  │      │ idle/1  │      │ idle/2  │
//!   └─────────┘      └─────────┘      └─────────┘
//!        │                │                │
//!        ▼                ▼                ▼
//!   ┌─────────────────────────────────────────────┐
//!   │  Loop: enable_irq → halt → disable_irq →   │
//!   │        schedule() → repeat                 │
//!   └─────────────────────────────────────────────┘
//! ```
//!
//! ## Invariantes
//!
//! - Idle task NUNCA entra na runqueue
//! - Idle task NUNCA migra para outra CPU
//! - Idle task NUNCA morre

use crate::arch::Cpu;
use crate::rmm::addr::VirtAddr;
use crate::sched::task::context::CpuContext;
use crate::sched::task::{Task, TaskState};
use crate::sys::types::Tid;
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};

use super::per_cpu::{this_cpu_id, CpuState, CPUS, MAX_CPUS};

/// Flags indicando quais CPUs têm idle task inicializada
static IDLE_INITIALIZED: [AtomicBool; MAX_CPUS] = {
    const INIT: AtomicBool = AtomicBool::new(false);
    [INIT; MAX_CPUS]
};

// =============================================================================
// ENTRY POINT
// =============================================================================

/// Entry point da idle task
#[no_mangle]
pub extern "C" fn idle_task_entry() -> ! {
    let cpu_id = this_cpu_id();
    crate::kdebug!("(Idle) Idle task CPU", cpu_id as u64, "iniciada");

    loop {
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();

        {
            if let Some(ref mut cpu_data) = *CPUS[cpu_id].lock() {
                cpu_data.state = CpuState::Idle;
            }
        }

        super::scheduler::schedule();
    }
}

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa idle task para uma CPU
pub fn init_idle_task_for_cpu(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        crate::kerror!("(Idle) CPU ID", cpu_id as u64, "inválido!");
        return;
    }

    if IDLE_INITIALIZED[cpu_id].swap(true, Ordering::SeqCst) {
        crate::kwarn!("(Idle) Idle task CPU", cpu_id as u64, "já existe!");
        return;
    }

    crate::kinfo!("(Idle) Criando idle/", cpu_id as u64);

    // Aloca stack (16KB)
    let stack_size = 16 * 1024;
    let stack_layout = alloc::alloc::Layout::from_size_align(stack_size, 16).unwrap();
    let stack_ptr = unsafe { alloc::alloc::alloc_zeroed(stack_layout) };

    if stack_ptr.is_null() {
        panic!("(Idle) Falha ao alocar stack para CPU {}!", cpu_id);
    }

    let stack_top = (stack_ptr as u64) + stack_size as u64;

    // TID: CPU 0 usa 0, APs usam 0x80000000 + cpu_id
    let tid = if cpu_id == 0 {
        0
    } else {
        0x80000000 + cpu_id as u32
    };

    // Nome
    let mut name_buf = [0u8; 32];
    let prefix = b"idle/";
    name_buf[..5].copy_from_slice(prefix);
    if cpu_id < 10 {
        name_buf[5] = b'0' + cpu_id as u8;
    }

    let mut idle_task = Box::pin(Task {
        tid: Tid::new(tid),
        state: TaskState::Running,
        context: CpuContext::new(),
        kernel_stack: VirtAddr::new(stack_top),
        user_stack: VirtAddr::new(0),
        aspace: None,
        priority: 255,
        accounting: crate::sched::task::accounting::Accounting::new(),
        parent_id: None,
        exit_code: None,
        pending_signals: 0,
        blocked_signals: 0,
        name: name_buf,
        handle_table: crate::syscall::handle::table::HandleTable::new(),
        wake_at: None,
        heap_start: 0,
        heap_next: 0,
    });

    // Configura contexto
    unsafe {
        let task_mut = Pin::get_unchecked_mut(idle_task.as_mut());
        task_mut.context.setup(
            VirtAddr::new(idle_task_entry as *const () as u64),
            VirtAddr::new(stack_top),
        );
    }

    // Armazena na estrutura per-CPU
    {
        let mut guard = CPUS[cpu_id].lock();
        if let Some(ref mut cpu_data) = *guard {
            cpu_data.idle_task = Some(idle_task);
            cpu_data.state = CpuState::Idle;
        } else {
            panic!("(Idle) CPU {} não inicializada!", cpu_id);
        }
    }

    crate::kinfo!("(Idle) idle/", cpu_id as u64, "pronta");
}

/// Alias para BSP (CPU 0)
pub fn init_idle_task() {
    init_idle_task_for_cpu(0);
}

// =============================================================================
// VERIFICAÇÃO
// =============================================================================

/// Verifica se idle task da CPU está inicializada
pub fn is_initialized_for_cpu(cpu_id: usize) -> bool {
    cpu_id < MAX_CPUS && IDLE_INITIALIZED[cpu_id].load(Ordering::SeqCst)
}

/// Verifica se CPU 0 (BSP) tem idle task
pub fn is_initialized() -> bool {
    is_initialized_for_cpu(0)
}

/// Verifica se uma task é idle
pub fn is_idle_task(task: &Task) -> bool {
    let tid = task.tid.as_u32();
    tid == 0 || tid >= 0x80000000
}

// =============================================================================
// CONTEXT SWITCH
// =============================================================================

/// Switch para idle task da CPU
pub unsafe fn switch_to_idle_cpu(cpu_id: usize, old_ctx: *mut CpuContext) {
    let guard = CPUS[cpu_id].lock();
    if let Some(ref cpu_data) = *guard {
        if let Some(ref idle_task) = cpu_data.idle_task {
            let idle_ctx = &idle_task.context as *const CpuContext;
            drop(guard);
            crate::sched::task::context::switch(&mut *old_ctx, &*idle_ctx);
            return;
        }
    }
    panic!("(Idle) Idle task não existe para CPU {}!", cpu_id);
}

/// Obtém contexto da idle task
pub unsafe fn get_idle_context_cpu(cpu_id: usize) -> *mut CpuContext {
    let mut guard = CPUS[cpu_id].lock();
    if let Some(ref mut cpu_data) = *guard {
        if let Some(ref mut idle_task) = cpu_data.idle_task {
            return &mut Pin::get_unchecked_mut(idle_task.as_mut()).context as *mut CpuContext;
        }
    }
    panic!("(Idle) Idle task não existe para CPU {}!", cpu_id);
}
