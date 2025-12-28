# Guia de Implementação: `sched/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Scheduler e gerenciamento de tarefas. Decide qual task roda em qual CPU e quando.

---

## 2. ESTRUTURA

```
sched/
├── mod.rs              ✅ JÁ EXISTE
├── task/
│   ├── mod.rs
│   ├── state.rs        → Estados de task
│   ├── thread.rs       → Thread Control Block
│   └── exit.rs         → Cleanup de task
├── context/
│   ├── mod.rs
│   └── switch.rs       → Context switch
├── scheduler/
│   ├── mod.rs
│   ├── policy.rs       → Algoritmos (RR, Priority)
│   ├── runqueue.rs     → Fila de prontos
│   └── load.rs         → Balanceamento
├── exec/
│   ├── mod.rs
│   ├── elf/
│   │   └── mod.rs      → ELF loader
│   ├── spawn/
│   │   ├── mod.rs
│   │   └── spawn.rs    → Criação de processo
│   └── interp/
│       ├── mod.rs
│       └── script.rs   → Scripts (#!)
├── signal/
│   ├── mod.rs
│   ├── delivery.rs     → Entrega de sinais
│   └── handler.rs      → Signal handlers
└── wait/
    ├── mod.rs
    └── waitqueue.rs    → Wait queues
```

---

## 3. REGRAS

### ❌ NUNCA:
- Usar `f32`/`f64`
- Usar `unwrap()`
- Acessar runqueue sem lock
- Fazer context switch com interrupções habilitadas

### ✅ SEMPRE:
- Desabilitar interrupções durante context switch
- Usar Pin<Box<Task>> para evitar moves
- Salvar/restaurar TODOS os registradores (incluindo FPU!)

---

## 4. IMPLEMENTAÇÕES

### 4.1 `task/state.rs`

```rust
//! Estados de task

/// Estado de uma task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Recém criada, não executou ainda
    Created,
    /// Pronta para executar
    Ready,
    /// Executando em alguma CPU
    Running,
    /// Bloqueada esperando algo
    Blocked,
    /// Terminada, esperando cleanup
    Zombie,
    /// Morta, pode ser liberada
    Dead,
}

impl TaskState {
    /// Verifica se pode ser escalonada
    pub const fn is_runnable(self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }
}
```

### 4.2 `task/thread.rs`

```rust
//! Thread Control Block

use crate::sys::types::Tid;
use crate::mm::VirtAddr;
use super::state::TaskState;
use super::super::context::CpuContext;

/// Task ID counter
static NEXT_TID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Thread Control Block
pub struct Task {
    /// ID único
    pub tid: Tid,
    /// Estado atual
    pub state: TaskState,
    /// Contexto de CPU salvo
    pub context: CpuContext,
    /// Stack pointer do kernel
    pub kernel_stack: VirtAddr,
    /// Stack pointer do usuário
    pub user_stack: VirtAddr,
    /// Prioridade (0 = maior)
    pub priority: u8,
    /// Nome (debug)
    pub name: [u8; 32],
}

impl Task {
    /// Cria nova task
    pub fn new(name: &str) -> Self {
        let tid = Tid::new(NEXT_TID.inc() as u32);
        let mut name_buf = [0u8; 32];
        let len = name.len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        Self {
            tid,
            state: TaskState::Created,
            context: CpuContext::new(),
            kernel_stack: VirtAddr::new(0),
            user_stack: VirtAddr::new(0),
            priority: 128, // Prioridade média
            name: name_buf,
        }
    }
    
    /// Marca como pronta
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready;
    }
    
    /// Marca como bloqueada
    pub fn set_blocked(&mut self) {
        self.state = TaskState::Blocked;
    }
}
```

### 4.3 `context/switch.rs`

```rust
//! Context switch

use crate::mm::VirtAddr;

/// Contexto de CPU (registradores salvos)
#[repr(C)]
pub struct CpuContext {
    // Callee-saved registers (SysV ABI)
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    // Stack pointer
    pub rsp: u64,
    
    // Instruction pointer (return address)
    pub rip: u64,
    
    // FPU/SSE state (512 bytes, 16-aligned)
    pub fpu_state: FpuState,
}

/// Estado FPU (área FXSAVE)
#[repr(C, align(16))]
pub struct FpuState {
    data: [u8; 512],
}

impl FpuState {
    pub const fn new() -> Self {
        Self { data: [0; 512] }
    }
    
    /// Salva estado FPU atual
    pub fn save(&mut self) {
        // SAFETY: fxsave é seguro com buffer alinhado
        unsafe {
            core::arch::asm!(
                "fxsave [{}]",
                in(reg) self.data.as_mut_ptr(),
                options(nostack)
            );
        }
    }
    
    /// Restaura estado FPU
    pub fn restore(&self) {
        // SAFETY: fxrstor é seguro com buffer válido
        unsafe {
            core::arch::asm!(
                "fxrstor [{}]",
                in(reg) self.data.as_ptr(),
                options(nostack)
            );
        }
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
            rip: 0,
            fpu_state: FpuState::new(),
        }
    }
    
    /// Configura para iniciar em função específica
    pub fn setup(&mut self, entry: VirtAddr, stack: VirtAddr) {
        self.rip = entry.as_u64();
        self.rsp = stack.as_u64();
        self.rbp = 0;
    }
}

/// Realiza context switch entre duas tasks
/// 
/// # Safety
/// 
/// - Interrupções devem estar desabilitadas
/// - old e new devem ser ponteiros válidos
pub unsafe fn switch(old: &mut CpuContext, new: &CpuContext) {
    // Salvar FPU do contexto antigo
    old.fpu_state.save();
    
    // Chamar assembly de switch
    context_switch_asm(
        old as *mut CpuContext as u64,
        new as *const CpuContext as u64,
    );
    
    // Restaurar FPU do novo contexto
    // (acontece quando voltamos a ser o "new")
}

extern "C" {
    fn context_switch_asm(old: u64, new: u64);
}
```

### 4.4 `scheduler/runqueue.rs`

```rust
//! Fila de tasks prontas

use alloc::collections::VecDeque;
use alloc::boxed::Box;
use core::pin::Pin;
use crate::sync::Spinlock;
use super::super::task::Task;

/// Fila de execução
pub struct RunQueue {
    /// Tasks prontas (FIFO simples por enquanto)
    queue: VecDeque<Pin<Box<Task>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    
    /// Adiciona task à fila
    pub fn push(&mut self, task: Pin<Box<Task>>) {
        self.queue.push_back(task);
    }
    
    /// Remove próxima task
    pub fn pop(&mut self) -> Option<Pin<Box<Task>>> {
        self.queue.pop_front()
    }
    
    /// Número de tasks na fila
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// Verifica se está vazia
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Runqueue global (TODO: per-CPU)
pub static RUNQUEUE: Spinlock<RunQueue> = Spinlock::new(RunQueue::new());
```

### 4.5 `scheduler/mod.rs`

```rust
//! Scheduler principal

use crate::sync::Spinlock;
use crate::arch::Cpu;
use super::task::Task;
use super::runqueue::RUNQUEUE;
use alloc::boxed::Box;
use core::pin::Pin;

/// Task atualmente executando (per-CPU no futuro)
static CURRENT: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

/// Inicializa o scheduler
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");
    // Criar idle task
    // TODO: criar idle task que só faz hlt
}

/// Retorna task atual
pub fn current() -> Option<*const Task> {
    CURRENT.lock().as_ref().map(|t| t.as_ref().get_ref() as *const Task)
}

/// Adiciona task à fila de execução
pub fn enqueue(task: Pin<Box<Task>>) {
    RUNQUEUE.lock().push(task);
}

/// Seleciona próxima task para executar
pub fn pick_next() -> Option<Pin<Box<Task>>> {
    RUNQUEUE.lock().pop()
}

/// Yield: cede CPU voluntariamente
pub fn yield_now() {
    Cpu::disable_interrupts();
    schedule();
    Cpu::enable_interrupts();
}

/// Função principal de escalonamento
pub fn schedule() {
    // Pegar próxima task
    let next = match pick_next() {
        Some(t) => t,
        None => return, // Sem tasks, continuar na atual
    };
    
    // Trocar contexto
    let mut current_guard = CURRENT.lock();
    if let Some(ref mut current) = *current_guard {
        // Salvar task atual de volta na fila
        let old_task = current_guard.take().unwrap();
        RUNQUEUE.lock().push(old_task);
    }
    
    *current_guard = Some(next);
    
    // TODO: fazer context switch real
}

/// Loop principal do scheduler (nunca retorna)
pub fn run() -> ! {
    loop {
        schedule();
        
        // Se não há tasks, esperar interrupção
        if RUNQUEUE.lock().is_empty() {
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}
```

### 4.6 `exec/spawn/spawn.rs`

```rust
//! Criação de processos

use crate::sys::{KernelError, KernelResult};
use crate::sys::types::Pid;
use crate::mm::VirtAddr;
use alloc::boxed::Box;
use core::pin::Pin;

/// Erro de execução
#[derive(Debug, Clone, Copy)]
pub enum ExecError {
    NotFound,
    InvalidFormat,
    OutOfMemory,
    PermissionDenied,
}

impl From<ExecError> for KernelError {
    fn from(e: ExecError) -> Self {
        match e {
            ExecError::NotFound => KernelError::NotFound,
            ExecError::InvalidFormat => KernelError::InvalidArgument,
            ExecError::OutOfMemory => KernelError::OutOfMemory,
            ExecError::PermissionDenied => KernelError::PermissionDenied,
        }
    }
}

/// Cria novo processo a partir de executável
pub fn spawn(path: &str) -> Result<Pid, ExecError> {
    crate::kinfo!("Spawning:", path.as_ptr() as u64);
    
    // 1. Abrir arquivo
    // TODO: usar VFS
    
    // 2. Carregar ELF
    // TODO: parsear headers, mapear segmentos
    
    // 3. Criar address space
    // TODO: criar page tables
    
    // 4. Criar task
    let mut task = crate::sched::task::Task::new(path);
    
    // 5. Configurar entry point
    // TODO: pegar do ELF header
    let entry = VirtAddr::new(0x400000); // placeholder
    let stack = VirtAddr::new(0x7FFF_FFFF_0000); // placeholder
    task.context.setup(entry, stack);
    
    // 6. Marcar como pronta
    task.set_ready();
    let pid = Pid::new(task.tid.as_u32());
    
    // 7. Adicionar ao scheduler
    crate::sched::scheduler::enqueue(Box::pin(task));
    
    Ok(pid)
}
```

### 4.7 `wait/waitqueue.rs`

```rust
//! Wait queues para bloqueio

use alloc::collections::VecDeque;
use crate::sync::Spinlock;
use crate::sys::types::Tid;

/// Wait queue - threads esperando evento
pub struct WaitQueue {
    waiters: Spinlock<VecDeque<Tid>>,
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            waiters: Spinlock::new(VecDeque::new()),
        }
    }
    
    /// Adiciona thread atual à espera
    pub fn wait(&self) {
        // TODO: pegar TID atual
        // TODO: marcar task como Blocked
        // TODO: adicionar à fila
        // TODO: chamar schedule()
    }
    
    /// Acorda uma thread
    pub fn wake_one(&self) {
        let mut waiters = self.waiters.lock();
        if let Some(_tid) = waiters.pop_front() {
            // TODO: marcar task como Ready
            // TODO: adicionar ao runqueue
        }
    }
    
    /// Acorda todas as threads
    pub fn wake_all(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(_tid) = waiters.pop_front() {
            // TODO: acordar cada uma
        }
    }
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `task/state.rs` - Estados
2. `context/switch.rs` - Contexto e FPU
3. `task/thread.rs` - TCB
4. `scheduler/runqueue.rs` - Fila
5. `scheduler/mod.rs` - Core scheduler
6. `wait/waitqueue.rs` - Blocking
7. `exec/spawn/spawn.rs` - Criação
8. `exec/elf/mod.rs` - ELF loader

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::arch` (Cpu)
- `crate::mm` (VirtAddr)
- `crate::sync`
- `crate::sys`
- `crate::klib`

NÃO pode importar de:
- `crate::fs` (exceto em exec/)
- `crate::ipc`

---

## 7. CHECKLIST

- [ ] FPU state é salvo/restaurado em context switch
- [ ] Interrupções desabilitadas durante switch
- [ ] Tasks usam Pin<Box<>> para evitar moves
- [ ] Runqueue protegida por lock
- [ ] ELF loader valida magic number
