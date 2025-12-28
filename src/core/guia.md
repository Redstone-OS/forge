# Guia de Implementação: `core/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE. Não improvise. Não adicione nada que não esteja aqui.

---

## 1. PROPÓSITO DESTE MÓDULO

O módulo `core/` é o **núcleo lógico** do kernel. Contém inicialização, gerenciamento de objetos, tempo, SMP e trabalho diferido. É **agnóstico de hardware** — não conhece x86 ou ARM.

---

## 2. ESTRUTURA DE ARQUIVOS OBRIGATÓRIA

```
core/
├── mod.rs              ✅ JÁ EXISTE - NÃO MODIFICAR
├── boot/
│   ├── mod.rs          → Exporta submódulos
│   ├── entry.rs        → kernel_main()
│   ├── handoff.rs      → BootInfo do bootloader
│   ├── cmdline.rs      → Parser de argumentos
│   ├── panic.rs        → #[panic_handler]
│   └── initcall.rs     → Registro de init functions
├── object/
│   ├── mod.rs          → Sistema de objetos
│   ├── kobject.rs      → Trait base KernelObject
│   ├── handle.rs       → Handle opaco
│   ├── rights.rs       → Direitos (READ, WRITE, etc)
│   ├── refcount.rs     → Contagem de referência
│   └── dispatcher.rs   → Handle → Object
├── smp/
│   ├── mod.rs          → Multiprocessamento
│   ├── percpu.rs       → Variáveis por CPU
│   ├── bringup.rs      → Wake APs
│   ├── ipi.rs          → Inter-Processor Interrupts
│   └── topology.rs     → Cores, sockets, HT
├── time/
│   ├── mod.rs          → Tempo e timers
│   ├── clock.rs        → Wall clock
│   ├── jiffies.rs      → Ticks desde boot
│   ├── timer.rs        → Interface genérica
│   └── hrtimer.rs      → High-resolution timers
├── work/
│   ├── mod.rs          → Trabalho diferido
│   ├── workqueue.rs    → Filas de trabalho
│   ├── tasklet.rs      → Tasks pequenas
│   └── deferred.rs     → Execução posterior
├── power/
│   ├── mod.rs          → Gerenciamento de energia
│   ├── state.rs        → Estados (Running, Suspend)
│   ├── cpufreq.rs      → Frequência de CPU
│   ├── cpuidle.rs      → C-States
│   └── suspend.rs      → S3/S4
└── debug/
    ├── mod.rs          → Diagnóstico
    ├── klog.rs         → Sistema de logs
    ├── kdebug.rs       → Breakpoints
    ├── oops.rs         → Erros recuperáveis
    ├── stats.rs        → Contadores
    └── trace.rs        → Tracing
```

---

## 3. REGRAS INVIOLÁVEIS

### ❌ NUNCA FAZER:
- Usar `asm!` ou assembly (isso vai em `arch/`)
- Usar `f32` ou `f64`
- Usar `unwrap()` ou `expect()` fora de constantes
- Importar diretamente de `arch::x86_64::` (use apenas `arch::Cpu`)
- Acessar hardware diretamente

### ✅ SEMPRE FAZER:
- Usar traits para abstração de hardware
- Retornar `Result<T, Error>`
- Documentar funções públicas
- Manter agnóstico de arquitetura

---

## 4. IMPLEMENTAÇÃO DETALHADA

### 4.1 `boot/mod.rs`

```rust
//! Inicialização do kernel

pub mod cmdline;
pub mod entry;
pub mod handoff;
pub mod initcall;
pub mod panic;

pub use entry::kernel_main;
pub use handoff::BootInfo;
```

### 4.2 `boot/handoff.rs`

```rust
//! Estrutura de handoff do bootloader

/// Tipo de região de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryRegionType {
    /// Memória utilizável
    Usable = 0,
    /// Reservada pelo firmware
    Reserved = 1,
    /// ACPI reclaimable
    AcpiReclaimable = 2,
    /// ACPI NVS
    AcpiNvs = 3,
    /// Região com defeito
    BadMemory = 4,
    /// Código do bootloader (pode ser reclamado)
    BootloaderReclaimable = 5,
    /// Código do kernel
    KernelAndModules = 6,
    /// Framebuffer
    Framebuffer = 7,
}

/// Uma região de memória física
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

/// Informações do framebuffer
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bpp: u32,
}

/// Informações passadas pelo bootloader
/// 
/// # IMPORTANTE
/// 
/// Esta estrutura DEVE ser idêntica byte-a-byte à do bootloader!
/// Use apenas tipos primitivos e `#[repr(C)]`.
#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    /// Mapa de memória física
    pub memory_map: &'static [MemoryRegion],
    
    /// Framebuffer (pode ser None)
    pub framebuffer: Option<FramebufferInfo>,
    
    /// Endereço físico das tabelas ACPI (RSDP)
    pub acpi_rsdp: Option<u64>,
    
    /// Linha de comando do kernel
    pub cmdline: Option<&'static str>,
    
    /// Endereço físico do initramfs
    pub initramfs_addr: Option<u64>,
    
    /// Tamanho do initramfs
    pub initramfs_size: u64,
}
```

### 4.3 `boot/entry.rs`

```rust
//! Ponto de entrada do kernel

use crate::core::boot::BootInfo;

/// Ponto de entrada principal do kernel.
///
/// Chamado pelo `_start` em main.rs após setup inicial.
///
/// # Ordem de Inicialização
///
/// 1. Debug/Logging
/// 2. Memória (PMM → VMM → Heap)
/// 3. Interrupções (IDT, APIC)
/// 4. Scheduler
/// 5. Syscalls
/// 6. Drivers
/// 7. Filesystem
/// 8. Init process
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Inicializar logging primeiro
    crate::drivers::serial::init();
    crate::kinfo!("Forge kernel inicializando...");
    
    // 2. Inicializar memória
    unsafe { crate::mm::init(boot_info); }
    
    // 3. Inicializar interrupções
    unsafe { 
        crate::arch::x86_64::idt::init();
        crate::arch::x86_64::apic::init();
    }
    
    // 4. Inicializar scheduler
    crate::sched::init();
    
    // 5. Inicializar syscalls
    crate::syscall::init();
    
    // 6. Inicializar drivers
    crate::drivers::init();
    
    // 7. Inicializar filesystem
    crate::fs::init();
    
    // 8. Inicializar IPC
    crate::ipc::init();
    
    // 9. Inicializar módulos
    crate::module::init();
    
    crate::kinfo!("Kernel inicializado, buscando init...");
    
    // Carregar e executar init
    match crate::sched::exec::spawn("/system/core/init") {
        Ok(_pid) => {
            crate::kinfo!("Init spawned, entrando no scheduler...");
        }
        Err(e) => {
            crate::kerror!("Falha ao spawnar init:", e as u64);
            panic!("Não foi possível iniciar o processo init");
        }
    }
    
    // Entrar no loop do scheduler (nunca retorna)
    crate::sched::scheduler::run()
}
```

### 4.4 `boot/panic.rs`

```rust
//! Panic handler do kernel

use core::panic::PanicInfo;

/// Handler de panic do kernel
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Desabilitar interrupções imediatamente
    crate::arch::Cpu::disable_interrupts();
    
    crate::kerror!("=== KERNEL PANIC ===");
    
    if let Some(location) = info.location() {
        crate::kerror!("File:", location.file().as_ptr() as u64);
        crate::kerror!("Line:", location.line() as u64);
    }
    
    if let Some(msg) = info.message() {
        // Não podemos formatar facilmente, apenas indicar que há mensagem
        crate::kerror!("Panic message presente");
    }
    
    // Halt loop
    loop {
        crate::arch::Cpu::halt();
    }
}
```

### 4.5 `object/mod.rs`

```rust
//! Sistema de objetos do kernel

pub mod dispatcher;
pub mod handle;
pub mod kobject;
pub mod refcount;
pub mod rights;

pub use handle::Handle;
pub use kobject::KernelObject;
pub use refcount::RefCount;
pub use rights::Rights;
```

### 4.6 `object/handle.rs`

```rust
//! Handle opaco para userspace

/// Handle opaco que representa um objeto do kernel.
///
/// Userspace nunca vê ponteiros reais, apenas handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(u32);

impl Handle {
    /// Handle inválido/nulo
    pub const INVALID: Handle = Handle(0);
    
    /// Cria novo handle a partir de índice
    pub const fn new(index: u32) -> Self {
        Self(index)
    }
    
    /// Retorna o valor raw do handle
    pub const fn raw(&self) -> u32 {
        self.0
    }
    
    /// Verifica se é válido
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
```

### 4.7 `object/rights.rs`

```rust
//! Direitos de acesso a objetos

/// Direitos que um handle pode ter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rights(u32);

impl Rights {
    /// Sem direitos
    pub const NONE: Rights = Rights(0);
    
    /// Pode ler
    pub const READ: Rights = Rights(1 << 0);
    
    /// Pode escrever
    pub const WRITE: Rights = Rights(1 << 1);
    
    /// Pode executar
    pub const EXECUTE: Rights = Rights(1 << 2);
    
    /// Pode duplicar o handle
    pub const DUPLICATE: Rights = Rights(1 << 3);
    
    /// Pode transferir via IPC
    pub const TRANSFER: Rights = Rights(1 << 4);
    
    /// Pode criar handles derivados
    pub const GRANT: Rights = Rights(1 << 5);
    
    /// Todos os direitos
    pub const ALL: Rights = Rights(0x3F);
    
    /// Verifica se tem direito específico
    pub const fn has(&self, right: Rights) -> bool {
        (self.0 & right.0) == right.0
    }
    
    /// Combina direitos
    pub const fn union(self, other: Rights) -> Rights {
        Rights(self.0 | other.0)
    }
    
    /// Interseção de direitos
    pub const fn intersect(self, other: Rights) -> Rights {
        Rights(self.0 & other.0)
    }
}
```

### 4.8 `object/kobject.rs`

```rust
//! Trait base para objetos do kernel

use super::Rights;

/// Tipos de objetos do kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectType {
    None = 0,
    Process = 1,
    Thread = 2,
    Vmo = 3,        // Virtual Memory Object
    Port = 4,       // IPC Port
    Channel = 5,    // IPC Channel
    Event = 6,
    Timer = 7,
    Interrupt = 8,
    Pager = 9,
}

/// Trait que todo objeto do kernel deve implementar
pub trait KernelObject: Send + Sync {
    /// Tipo do objeto
    fn object_type(&self) -> ObjectType;
    
    /// Direitos padrão para este tipo de objeto
    fn default_rights(&self) -> Rights;
    
    /// Chamado quando última referência é liberada
    fn on_destroy(&mut self) {}
}
```

### 4.9 `debug/klog.rs`

```rust
//! Sistema de logging do kernel

use crate::drivers::serial;

/// Nível de log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Emite uma linha de log
pub fn log(level: LogLevel, message: &str) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG] ",
        LogLevel::Info => "[INFO]  ",
        LogLevel::Warn => "[WARN]  ",
        LogLevel::Error => "[ERROR] ",
    };
    
    serial::write_str(prefix);
    serial::write_str(message);
    serial::write_str("\n");
}

/// Emite log com valor hexadecimal
pub fn log_hex(level: LogLevel, message: &str, value: u64) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG] ",
        LogLevel::Info => "[INFO]  ",
        LogLevel::Warn => "[WARN]  ",
        LogLevel::Error => "[ERROR] ",
    };
    
    serial::write_str(prefix);
    serial::write_str(message);
    serial::write_str(" 0x");
    serial::write_hex(value);
    serial::write_str("\n");
}

// Macros de conveniência
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Info,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Info,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Warn,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Warn,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Error,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Error,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Debug,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Debug,
            $msg,
            $val as u64
        )
    };
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `debug/klog.rs` - Logging primeiro (necessário para debug)
2. `boot/handoff.rs` - Estrutura BootInfo
3. `boot/panic.rs` - Panic handler
4. `object/rights.rs` - Direitos
5. `object/handle.rs` - Handle opaco
6. `object/kobject.rs` - Trait base
7. `object/refcount.rs` - Contagem de referência
8. `object/dispatcher.rs` - Lookup de objetos
9. `boot/entry.rs` - kernel_main
10. `smp/percpu.rs` - Variáveis por CPU
11. `time/jiffies.rs` - Contador simples
12. `work/workqueue.rs` - Filas de trabalho

---

## 6. DEPENDÊNCIAS

Este módulo pode importar de:
- `crate::arch` (apenas via traits, ex: `arch::Cpu`)
- `crate::klib`
- `crate::sync`
- `crate::drivers::serial` (apenas para logging)

Este módulo NÃO pode importar de:
- `crate::mm` (exceto em `boot/entry.rs` para inicialização)
- `crate::sched` (exceto em `boot/entry.rs`)
- `crate::fs`
- `crate::ipc`

---

## 7. CHECKLIST FINAL

- [ ] Nenhum assembly direto (usa `arch::Cpu`)
- [ ] Nenhum `unwrap()` ou `expect()`
- [ ] Todo `unsafe` tem `// SAFETY:`
- [ ] Macros de log funcionam (`kinfo!`, `kerror!`, etc)
- [ ] BootInfo é `#[repr(C)]`
- [ ] Panic handler não aloca memória
