//! Idle Task - Tarefa ociosa dedicada como fallback permanente
//!
//! A idle task é mantida em uma variável estática separada (IDLE_TASK) e
//! NUNCA é removida. Quando não há tasks prontas, o sistema sempre volta
//! para a idle task de forma segura.

use crate::arch::Cpu;
use crate::mm::VirtAddr;
use crate::sched::task::context::CpuContext;
use crate::sched::task::{Task, TaskState};
use crate::sync::Spinlock;
use crate::sys::types::Tid;
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};

/// Flag indicando se a idle task foi inicializada
static IDLE_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// A IDLE TASK permanente - NUNCA é removida daqui
/// Esta é a diferença crucial: a idle task tem sua própria "casa" permanente
pub static IDLE_TASK: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

/// Retorna um ponteiro mutável para o contexto da idle task
///
/// # Safety
/// O chamador deve garantir que a idle task está inicializada
pub unsafe fn get_idle_context() -> *mut CpuContext {
    let mut guard = IDLE_TASK.lock();
    if let Some(ref mut task) = *guard {
        &mut Pin::get_unchecked_mut(task.as_mut()).context as *mut CpuContext
    } else {
        panic!("(Idle) get_idle_context chamado sem idle task inicializada!");
    }
}

/// Entry point da idle task - loop infinito de espera
#[no_mangle]
pub extern "C" fn idle_task_entry() -> ! {
    crate::kinfo!("(Idle) Idle task iniciada (TID 0)");

    let mut idle_count: u64 = 0;

    loop {
        // Habilita interrupções e espera próximo evento
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();

        idle_count = idle_count.wrapping_add(1);

        // Log periódico a cada 1000 iterações
        if idle_count % 1000 == 0 {
            crate::kdebug!("(Idle) Ciclos:", idle_count);
        }

        // Verifica se há tasks prontas e chama schedule
        super::scheduler::schedule();
        // Sempre retorna aqui quando não há mais tasks
    }
}

/// Cria e inicializa a idle task
///
/// A idle task é criada e armazenada em IDLE_TASK (não em CURRENT).
/// Isso garante que ela SEMPRE está disponível como fallback.
pub fn init_idle_task() {
    if IDLE_INITIALIZED.swap(true, Ordering::SeqCst) {
        crate::kwarn!("(Idle) init_idle_task chamado mais de uma vez!");
        return;
    }

    crate::kinfo!("(Idle) Criando idle task...");

    // Aloca stack para a idle task (16KB)
    let stack_size = 16 * 1024;
    let stack_layout = alloc::alloc::Layout::from_size_align(stack_size, 16).unwrap();
    let stack_ptr = unsafe { alloc::alloc::alloc_zeroed(stack_layout) };

    if stack_ptr.is_null() {
        panic!("(Idle) Falha ao alocar stack para idle task!");
    }

    let stack_top = (stack_ptr as u64) + stack_size as u64;

    // Cria a task manualmente com TID 0
    let mut name_buf = [0u8; 32];
    let name = b"idle";
    name_buf[..4].copy_from_slice(name);

    let mut idle_task = Box::pin(Task {
        tid: Tid::new(0),
        state: TaskState::Running, // Idle sempre está "pronta"
        context: CpuContext::new(),
        kernel_stack: VirtAddr::new(stack_top),
        user_stack: VirtAddr::new(0),
        aspace: None,  // Usa espaço de endereçamento do kernel
        priority: 255, // Menor prioridade
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

    // Configura o contexto para iniciar em idle_task_entry
    unsafe {
        let task_mut = Pin::get_unchecked_mut(idle_task.as_mut());
        task_mut.context.setup(
            VirtAddr::new(idle_task_entry as *const () as u64),
            VirtAddr::new(stack_top),
        );
    }

    // Armazena em IDLE_TASK (local permanente!)
    {
        let mut idle_guard = IDLE_TASK.lock();
        *idle_guard = Some(idle_task);
    }

    crate::kinfo!("(Idle) Idle task criada e armazenada em IDLE_TASK");
}

/// Verifica se a idle task está inicializada
pub fn is_initialized() -> bool {
    IDLE_INITIALIZED.load(Ordering::SeqCst)
}

/// Verifica se uma task é a idle task (TID 0)
pub fn is_idle_task(task: &Task) -> bool {
    task.tid.as_u32() == 0
}

/// Faz o switch para a idle task quando não há mais tasks
///
/// # Safety
/// - Interrupções devem estar desabilitadas
/// - old_ctx deve ser um ponteiro válido para salvar o contexto atual
pub unsafe fn switch_to_idle(old_ctx: *mut CpuContext) {
    let idle_guard = IDLE_TASK.lock();
    if let Some(ref idle_task) = *idle_guard {
        let idle_ctx = &idle_task.context as *const CpuContext;

        crate::ktrace!("(Idle) Retornando para idle task");

        // Switch: salva contexto atual em old_ctx, restaura contexto da idle
        drop(idle_guard); // Libera o lock ANTES do switch
        crate::sched::task::context::switch(&mut *old_ctx, &*idle_ctx);

        // Retorna aqui quando a task for re-escalonada
        crate::ktrace!("(Idle) Task retomada do idle");
    } else {
        panic!("(Idle) switch_to_idle: idle task não inicializada!");
    }
}

/// Obtém o ponteiro para o contexto da idle task (sem lock - para leitura rápida)
///
/// # Safety
/// Só usar após init_idle_task() e com cuidado com concorrência
pub unsafe fn idle_context_ptr() -> *const CpuContext {
    let guard = IDLE_TASK.lock();
    if let Some(ref task) = *guard {
        &task.context as *const CpuContext
    } else {
        core::ptr::null()
    }
}
