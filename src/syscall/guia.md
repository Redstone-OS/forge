# Guia de Revisao: `syscall/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Interface única entre userspace (Ring 3) e kernel (Ring 0). Toda operação privilegiada passa por aqui.

---

## 2. ESTRUTURA

```
syscall/
├── mod.rs              ✅ JÁ EXISTE
├── abi/
│   ├── mod.rs
│   ├── args.rs         → SyscallArgs
│   ├── flags.rs        → Flags comuns
│   ├── types.rs        → Tipos compartilhados
│   └── version.rs      → Versão da ABI
├── dispatch/
│   ├── mod.rs
│   └── table.rs        → Tabela de handlers
├── error.rs            → SysError
├── numbers.rs          → Números de syscall (IMUTÁVEIS!)
├── handle/
│   ├── mod.rs
│   ├── table.rs        → HandleTable
│   └── rights.rs       → Rights
├── process/
│   ├── mod.rs
│   ├── lifecycle.rs    → exit, spawn, wait
│   └── info.rs         → getpid, etc
├── memory/
│   ├── mod.rs
│   └── alloc.rs        → mmap, munmap
├── fs/
│   ├── mod.rs
│   └── basic.rs        → open, read, write, close
├── ipc/
│   ├── mod.rs
│   └── port.rs         → port_create, send, recv
├── event/
│   ├── mod.rs
│   └── poll.rs         → poll
├── time/
│   ├── mod.rs
│   └── clock.rs        → clock_get, sleep
└── system/
    ├── mod.rs
    └── info.rs         → sysinfo
```

---

## 3. REGRAS

### ❌ NUNCA:
- Confiar em argumentos do userspace
- Derreferenciar ponteiros sem validar
- Retornar ponteiros de kernel
- Modificar números de syscall existentes

### ✅ SEMPRE:
- Validar TODOS os argumentos
- Usar handles (não ponteiros)
- Retornar erros apropriados
- Documentar convenção de chamada

---

## 4. IMPLEMENTAÇÕES

### 4.1 `numbers.rs`

```rust
//! Números de syscall - IMUTÁVEIS APÓS RELEASE!

// === Process ===
pub const SYS_EXIT: u64 = 0;
pub const SYS_SPAWN: u64 = 1;
pub const SYS_WAIT: u64 = 2;
pub const SYS_YIELD: u64 = 3;
pub const SYS_GETPID: u64 = 4;
pub const SYS_GETTID: u64 = 5;

// === Memory ===
pub const SYS_MMAP: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_MPROTECT: u64 = 12;

// === Handles ===
pub const SYS_CLOSE: u64 = 20;
pub const SYS_DUP: u64 = 21;

// === Filesystem ===
pub const SYS_OPEN: u64 = 30;
pub const SYS_READ: u64 = 31;
pub const SYS_WRITE: u64 = 32;
pub const SYS_SEEK: u64 = 33;
pub const SYS_STAT: u64 = 34;

// === IPC ===
pub const SYS_PORT_CREATE: u64 = 40;
pub const SYS_PORT_SEND: u64 = 41;
pub const SYS_PORT_RECV: u64 = 42;
pub const SYS_CHANNEL_CREATE: u64 = 43;

// === Events ===
pub const SYS_POLL: u64 = 50;

// === Time ===
pub const SYS_CLOCK_GET: u64 = 60;
pub const SYS_SLEEP: u64 = 61;

// === System ===
pub const SYS_SYSINFO: u64 = 70;
pub const SYS_DEBUG: u64 = 71;

// === Último número válido ===
pub const SYS_MAX: u64 = 71;
```

### 4.2 `error.rs`

```rust
//! Códigos de erro de syscall

/// Erro de syscall
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum SysError {
    /// Sucesso (não é erro)
    Success = 0,
    /// Syscall não existe
    InvalidSyscall = -1,
    /// Argumento inválido
    InvalidArgument = -2,
    /// Handle inválido
    InvalidHandle = -3,
    /// Permissão negada
    PermissionDenied = -4,
    /// Não encontrado
    NotFound = -5,
    /// Já existe
    AlreadyExists = -6,
    /// Sem memória
    OutOfMemory = -7,
    /// Recurso ocupado
    Busy = -8,
    /// Timeout
    Timeout = -9,
    /// Interrompido
    Interrupted = -10,
    /// Buffer pequeno
    BufferTooSmall = -11,
    /// Novamente (não-bloqueante)
    Again = -12,
    /// Fim de arquivo
    EndOfFile = -13,
    /// Erro interno
    Internal = -99,
}

impl SysError {
    /// Converte para código
    pub const fn as_code(self) -> i64 {
        self as i64
    }
}

/// Resultado de syscall
pub type SysResult<T> = Result<T, SysError>;

/// Converte Result para retorno de syscall (i64)
pub fn to_syscall_result(result: SysResult<u64>) -> i64 {
    match result {
        Ok(val) => val as i64,
        Err(e) => e.as_code(),
    }
}
```

### 4.3 `abi/args.rs`

```rust
//! Argumentos de syscall

/// Argumentos passados para syscall
#[repr(C)]
pub struct SyscallArgs {
    pub num: u64,   // RAX - número da syscall
    pub arg0: u64,  // RDI
    pub arg1: u64,  // RSI
    pub arg2: u64,  // RDX
    pub arg3: u64,  // R10 (não RCX - destruído por syscall)
    pub arg4: u64,  // R8
    pub arg5: u64,  // R9
}

impl SyscallArgs {
    /// Interpreta arg como handle
    pub fn as_handle(&self, index: usize) -> crate::core::object::Handle {
        let raw = match index {
            0 => self.arg0,
            1 => self.arg1,
            2 => self.arg2,
            3 => self.arg3,
            4 => self.arg4,
            5 => self.arg5,
            _ => 0,
        };
        crate::core::object::Handle::new(raw as u32)
    }
    
    /// Interpreta arg como ponteiro de userspace
    /// 
    /// # Safety
    /// 
    /// Caller deve validar que ponteiro está em userspace
    pub unsafe fn as_user_ptr<T>(&self, index: usize) -> *const T {
        let raw = match index {
            0 => self.arg0,
            1 => self.arg1,
            2 => self.arg2,
            3 => self.arg3,
            4 => self.arg4,
            5 => self.arg5,
            _ => 0,
        };
        raw as *const T
    }
    
    /// Interpreta arg como ponteiro mutável de userspace
    pub unsafe fn as_user_ptr_mut<T>(&self, index: usize) -> *mut T {
        self.as_user_ptr::<T>(index) as *mut T
    }
}
```

### 4.4 `abi/version.rs`

```rust
//! Versionamento de ABI

/// Versão da ABI de syscalls
pub const ABI_VERSION: u32 = 1;

/// Magic number para validação
pub const REDSTONE_MAGIC: u32 = 0x5253444E; // "RSDN"
```

### 4.5 `dispatch/table.rs`

```rust
//! Tabela de dispatch de syscalls

use super::super::abi::SyscallArgs;
use super::super::error::{SysError, SysResult};
use super::super::numbers::*;

/// Tipo de handler de syscall
pub type SyscallHandler = fn(&SyscallArgs) -> SysResult<u64>;

/// Tamanho da tabela
pub const TABLE_SIZE: usize = 128;

/// Tabela de handlers
static SYSCALL_TABLE: [SyscallHandler; TABLE_SIZE] = {
    let mut table: [SyscallHandler; TABLE_SIZE] = [sys_invalid; TABLE_SIZE];
    
    // Process
    table[SYS_EXIT as usize] = sys_exit;
    table[SYS_YIELD as usize] = sys_yield;
    table[SYS_GETPID as usize] = sys_getpid;
    
    // Memory
    // table[SYS_MMAP as usize] = sys_mmap;
    
    // Handle
    table[SYS_CLOSE as usize] = sys_close;
    
    // FS
    table[SYS_OPEN as usize] = sys_open;
    table[SYS_READ as usize] = sys_read;
    table[SYS_WRITE as usize] = sys_write;
    
    // IPC
    // table[SYS_PORT_CREATE as usize] = sys_port_create;
    
    // Time
    // table[SYS_CLOCK_GET as usize] = sys_clock_get;
    
    // System
    table[SYS_DEBUG as usize] = sys_debug;
    
    table
};

/// Dispatcher principal
pub fn dispatch(args: &SyscallArgs) -> i64 {
    let num = args.num as usize;
    
    if num >= TABLE_SIZE {
        return SysError::InvalidSyscall.as_code();
    }
    
    let handler = SYSCALL_TABLE[num];
    let result = handler(args);
    
    super::super::error::to_syscall_result(result)
}

// === Handlers ===

fn sys_invalid(_args: &SyscallArgs) -> SysResult<u64> {
    Err(SysError::InvalidSyscall)
}

fn sys_exit(args: &SyscallArgs) -> SysResult<u64> {
    let code = args.arg0 as i32;
    crate::kinfo!("sys_exit:", code as u64);
    // TODO: terminar processo atual
    loop { crate::arch::Cpu::halt(); }
}

fn sys_yield(_args: &SyscallArgs) -> SysResult<u64> {
    crate::sched::scheduler::yield_now();
    Ok(0)
}

fn sys_getpid(_args: &SyscallArgs) -> SysResult<u64> {
    // TODO: retornar PID real
    Ok(1)
}

fn sys_close(args: &SyscallArgs) -> SysResult<u64> {
    let handle = args.as_handle(0);
    if !handle.is_valid() {
        return Err(SysError::InvalidHandle);
    }
    // TODO: fechar handle
    Ok(0)
}

fn sys_open(args: &SyscallArgs) -> SysResult<u64> {
    // TODO: validar ponteiro de userspace
    // TODO: abrir arquivo
    Err(SysError::NotFound)
}

fn sys_read(args: &SyscallArgs) -> SysResult<u64> {
    let handle = args.as_handle(0);
    if !handle.is_valid() {
        return Err(SysError::InvalidHandle);
    }
    // TODO: ler do handle
    Err(SysError::Internal)
}

fn sys_write(args: &SyscallArgs) -> SysResult<u64> {
    let handle = args.as_handle(0);
    // Caso especial: stdout
    if handle.raw() == 1 {
        let ptr = args.arg1 as *const u8;
        let len = args.arg2 as usize;
        
        // TODO: validar ponteiro de userspace!
        for i in 0..len {
            let byte = unsafe { *ptr.add(i) };
            crate::drivers::serial::write_byte(byte);
        }
        return Ok(len as u64);
    }
    
    if !handle.is_valid() {
        return Err(SysError::InvalidHandle);
    }
    
    Err(SysError::Internal)
}

fn sys_debug(args: &SyscallArgs) -> SysResult<u64> {
    crate::kinfo!("sys_debug:", args.arg0);
    crate::kinfo!("  arg1:", args.arg1);
    crate::kinfo!("  arg2:", args.arg2);
    Ok(0)
}
```

### 4.6 `dispatch/mod.rs`

```rust
//! Dispatch de syscalls

pub mod table;

use super::abi::SyscallArgs;

/// Dispatcher principal - chamado pelo entry point assembly
#[no_mangle]
pub extern "C" fn syscall_dispatcher(args: &SyscallArgs) -> i64 {
    table::dispatch(args)
}
```

### 4.7 `handle/table.rs`

```rust
//! Tabela de handles por processo

use crate::core::object::{Handle, KernelObject};
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// Entrada na tabela de handles
struct HandleEntry {
    object: *mut dyn KernelObject,
    rights: HandleRights,
}

/// Direitos do handle
#[derive(Debug, Clone, Copy)]
pub struct HandleRights(u32);

impl HandleRights {
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXECUTE: Self = Self(4);
    pub const ALL: Self = Self(7);
}

/// Tipo do handle
#[derive(Debug, Clone, Copy)]
pub enum HandleType {
    None,
    File,
    Port,
    Process,
    Thread,
    Event,
}

/// Tabela de handles
pub struct HandleTable {
    entries: Vec<Option<HandleEntry>>,
    next_free: usize,
}

impl HandleTable {
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(64);
        // Handle 0 é sempre None (inválido)
        entries.push(None);
        
        Self {
            entries,
            next_free: 1,
        }
    }
    
    /// Insere objeto e retorna handle
    pub fn insert(&mut self, object: *mut dyn KernelObject, rights: HandleRights) -> Handle {
        let entry = HandleEntry { object, rights };
        
        // Procurar slot livre
        for i in self.next_free..self.entries.len() {
            if self.entries[i].is_none() {
                self.entries[i] = Some(entry);
                self.next_free = i + 1;
                return Handle::new(i as u32);
            }
        }
        
        // Adicionar novo
        let index = self.entries.len();
        self.entries.push(Some(entry));
        self.next_free = index + 1;
        Handle::new(index as u32)
    }
    
    /// Busca objeto
    pub fn get(&self, handle: Handle) -> Option<*mut dyn KernelObject> {
        let index = handle.raw() as usize;
        self.entries.get(index)?.as_ref().map(|e| e.object)
    }
    
    /// Remove handle
    pub fn remove(&mut self, handle: Handle) -> bool {
        let index = handle.raw() as usize;
        if index >= self.entries.len() {
            return false;
        }
        
        if self.entries[index].is_some() {
            self.entries[index] = None;
            if index < self.next_free {
                self.next_free = index;
            }
            true
        } else {
            false
        }
    }
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `numbers.rs` - Números PRIMEIRO (imutáveis!)
2. `error.rs` - Códigos de erro
3. `abi/args.rs` - Argumentos
4. `abi/version.rs` - Versão
5. `dispatch/table.rs` - Tabela e handlers
6. `dispatch/mod.rs` - Entry point
7. `handle/table.rs` - HandleTable
8. Syscalls específicos (process/, fs/, etc)

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::arch` (Cpu)
- `crate::sync`
- `crate::mm`
- `crate::sched`
- `crate::fs`
- `crate::ipc`
- `crate::core::object`

---

## 7. CHECKLIST

- [ ] Números de syscall estão fixos
- [ ] Todos os handlers validam argumentos
- [ ] HandleTable protege handle 0
- [ ] Error codes são negativos
- [ ] Dispatcher retorna i64
