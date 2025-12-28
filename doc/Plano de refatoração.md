# 🔥 Plano de Refatoração Completo — Redstone OS Forge

> **Versão:** 2.0 — *Dezembro 2024*  
> **Objetivo:** Transformar o Forge de um kernel experimental básico em um kernel moderno, seguro e production-ready.

---

## 📋 Índice

1. [Veredito Executivo](#1-veredito-executivo)
2. [Princípios Inegociáveis](#2-princípios-inegociáveis)
3. [Análise de Aderência](#3-análise-de-aderência-aos-princípios)
4. [Mapa Arquitetural Completo](#4-mapa-arquitetural-completo)
5. [Análise por Módulo](#5-análise-por-módulo)
6. [Lacunas Críticas](#6-lacunas-críticas-identificadas)
7. [Plano de Implementação](#7-plano-de-implementação-por-fases)
8. [Requisitos por Arquivo](#8-requisitos-detalhados-por-arquivo)
9. [Critérios de Aceitação](#9-critérios-de-aceitação)
10. [Glossário Técnico](#10-glossário-técnico)

---

## 1. Veredito Executivo

### ✅ A Estrutura Está Aprovada

A reestruturação do `forge` **atinge o objetivo** de romper com o passado. A estrutura atual:

- **Não é cosmética**: Reflete uma mudança fundamental de filosofia
- **Permite "Guest with Badge"**: Módulos supervisionados com capabilities
- **Segue Micro-Modularidade**: Separação clara entre camadas
- **Evita armadilhas UNIX**: Não replica erros históricos do Linux/Windows

### ⚠️ Estado Atual: Esqueleto Funcional

| Aspecto | Nota | Observação |
|---------|------|------------|
| Arquitetura | **A** | Estrutura excelente, baseada em princípios modernos |
| Design | **A-** | Separação clara, capability-based pensado desde o início |
| Implementação | **C** | Muitos arquivos são apenas TODOs críticos |
| Documentação | **B+** | Boa documentação inline, falta especificação formal |

> **Diagnóstico**: A fundação está sólida. O trabalho agora é "preencher a carne" sem comprometer a arquitetura.

---

## 2. Princípios Inegociáveis

### 🛡️ Regra de Ouro
```
COMPATIBILIDADE SÓ SE CUSTO = 0
```

Se existe forma melhor, quebre compatibilidade. O kernel não carrega legado.

### 📜 Regras de Código

| Regra | Enforcement |
|-------|-------------|
| NUNCA `f32`/`f64` no kernel | ❌ SSE desabilitado no target spec |
| NUNCA `unwrap()`/`expect()` fora do boot | ✅ Auditoria obrigatória em CI |
| TODO bloco `unsafe` com `// SAFETY:` | ✅ Lint customizado |
| Retornar `Result<T, Error>` sempre | ✅ Clippy deny |
| Logging via macros centralizadas | ✅ `kinfo!`, `kwarn!`, `kerror!` |
| Kernel NUNCA depende de crates externas | ✅ Verificar `Cargo.toml` |

### 🏛️ Separação de Responsabilidades

```
┌─────────────────────────────────────────────────────────────────────┐
│                           USERSPACE (Ring 3)                        │
└─────────────────────────────────────────────────────────────────────┘
                              ↑ syscall ↓
┌─────────────────────────────────────────────────────────────────────┐
│ SYSCALL LAYER │ Única porta de entrada. Valida TUDO antes de       │
│               │ repassar para subsistemas. Hardcoded, imutável.     │
└─────────────────────────────────────────────────────────────────────┘
                              ↑ handles ↓
┌─────────────────────────────────────────────────────────────────────┐
│ CORE LAYER    │ Orquestração lógica. Scheduler, IPC, VFS, Security  │
│               │ Agnóstico de hardware. Nunca toca registradores.    │
└─────────────────────────────────────────────────────────────────────┘
                              ↑ traits ↓
┌─────────────────────────────────────────────────────────────────────┐
│ ARCH LAYER    │ HAL. Traduz conceitos abstratos para CPU específica.│
│ (x86_64)      │ Assembly, MSRs, Page Tables físicas. Isolado.       │
└─────────────────────────────────────────────────────────────────────┘
                              ↑ ABI ↓
┌─────────────────────────────────────────────────────────────────────┐
│ MODULE LAYER  │ Drivers carregáveis. Ring 0 supervisionado.         │
│ (ko files)    │ Sem acesso direto ao Core. Só via capability tokens.│
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Análise de Aderência aos Princípios

### ✅ Modularidade e Isolamento — Nota: A

| Critério | Status | Evidência |
|----------|--------|-----------|
| `arch/` isola código de CPU | ✅ | Assembly apenas em `arch/x86_64/` |
| `core/` é agnóstico de HW | ✅ | Nenhum `asm!` em `core/` |
| `module/` tem verificação | ✅ | `verifier.rs`, `sandbox.rs`, `capability.rs` existem |
| Drivers separados de Interfaces | ⚠️ | Drivers ainda em `src/drivers/`, idealmente seriam crates |

**Observação**: A pasta `src/module/` é a joia da coroa. Prova que o sistema foi desenhado para "Zero Trust" desde o início.

### ✅ Segurança e Type Safety — Nota: A-

| Critério | Status | Evidência |
|----------|--------|-----------|
| Uso de `Result` ao invés de panic | ✅ | Assinaturas não usam `unwrap()` |
| Capability-based (não ACL) | ✅ | `core/object/` com handles e rights |
| Handles opacos para userspace | ✅ | `Handle` é `u32` opaco |
| CSpace/CNode hierárquico | ❌ | **FALTA IMPLEMENTAR** |
| Revogação de capabilities | ❌ | **FALTA IMPLEMENTAR** |

**Observação**: O design de `core/object/` segue padrões modernos (Zircon/seL4), mas a implementação real de CSpace está ausente.

### ⚠️ Assincronismo — Nota: B em Design, C em Implementação

| Critério | Status | Evidência |
|----------|--------|-----------|
| Workqueues/Tasklets | ✅ | `core/work/` existe |
| `async/await` no kernel | ❌ | Nenhum `Future`, `Waker`, `Executor` |
| Drivers podem ser async | ❌ | `driver.rs` está vazio |
| IPC não-bloqueante integrado | ⚠️ | `recv()` retorna `Empty`, não bloqueia |

**Decisão Necessária**: O kernel vai adotar `async/await` nativo ou modelo tradicional de interrupção/callback?

### ⚠️ Maturidade do Código — Nota: Esqueleto

| Arquivo Crítico | Estado | Impacto |
|-----------------|--------|---------|
| `drivers/base/driver.rs` | **VAZIO** | Não existe contrato Driver↔Kernel |
| `sched/context/` | ⚠️ | Falta contexto FPU/SSE (corrompe apps) |
| `security/` | ⚠️ | Falta CSpace real |
| `ipc/` | ⚠️ | Sem integração com scheduler (busy wait) |

---

## 4. Mapa Arquitetural Completo

### 📁 Estrutura de Diretórios

```
src/
├── lib.rs              # Crate library entry
├── main.rs             # Binário: _start, stack, BSS zero
│
├── arch/               # 🔧 HAL - Hardware Abstraction Layer
│   ├── mod.rs
│   ├── traits/         # Contratos abstratos
│   │   ├── cpu.rs      # halt(), disable_interrupts()
│   │   └── mod.rs
│   └── x86_64/         # Implementação específica
│       ├── cpu.rs      # MSRs, CR0, CR3
│       ├── gdt.rs      # Segmentos (Kernel/User Code/Data)
│       ├── idt.rs      # Tabela de interrupções
│       ├── interrupts.rs # Handlers Rust
│       ├── memory.rs   # Setup inicial de paginação
│       ├── ports.rs    # inb/outb (IO Ports legadas)
│       ├── switch.s    # Context Switch Assembly
│       ├── syscall.rs  # LSTAR, STAR config
│       ├── syscall.s   # Trampolim user↔kernel
│       ├── acpi/       # Configuração de energia/HW
│       │   ├── dsdt.rs, fadt.rs, madt.rs
│       ├── apic/       # Controlador de interrupções
│       │   ├── ioapic.rs, lapic.rs
│       └── iommu/      # Isolamento de DMA
│           └── intel_vtd.rs
│
├── core/               # 🧠 Núcleo Lógico (Agnóstico de HW)
│   ├── boot/           # Inicialização
│   │   ├── cmdline.rs  # Parser de argumentos
│   │   ├── entry.rs    # kernel_main()
│   │   ├── handoff.rs  # BootInfo do bootloader
│   │   ├── initcall.rs # Registro auto de init funcs
│   │   └── panic.rs    # #[panic_handler]
│   ├── debug/          # Diagnóstico
│   │   ├── kdebug.rs   # Breakpoints, inspeção
│   │   ├── klog.rs     # Sistema de logs
│   │   ├── oops.rs     # Erros recuperáveis
│   │   ├── stats.rs    # Contadores globais
│   │   └── trace.rs    # Tracing de performance
│   ├── object/         # Gerenciamento de Recursos
│   │   ├── dispatcher.rs # Handle → Objeto real
│   │   ├── handle.rs   # Ponteiro seguro opaco
│   │   ├── kobject.rs  # Trait base (Process, Thread, VMO)
│   │   ├── refcount.rs # Arc manual
│   │   └── rights.rs   # READ, WRITE, EXECUTE
│   ├── power/          # Gestão de Energia
│   │   ├── cpufreq.rs  # Escalonamento de frequência
│   │   ├── cpuidle.rs  # C-States
│   │   ├── state.rs    # Máquina de estados
│   │   └── suspend.rs  # S3/S4
│   ├── smp/            # Multiprocessamento
│   │   ├── bringup.rs  # Wake APs
│   │   ├── ipi.rs      # Inter-Processor Interrupts
│   │   ├── percpu.rs   # Variáveis por CPU
│   │   └── topology.rs # Cores, Sockets, HT
│   ├── time/           # Relógio do Sistema
│   │   ├── clock.rs    # Wall Clock
│   │   ├── hrtimer.rs  # High-res timers
│   │   ├── jiffies.rs  # Ticks desde boot
│   │   └── timer.rs    # Interface genérica
│   └── work/           # Trabalho Diferido
│       ├── deferred.rs # Execução posterior
│       ├── tasklet.rs  # Tasks de alta prioridade
│       └── workqueue.rs # Filas de trabalho
│
├── drivers/            # 🔌 Drivers e Barramentos
│   ├── base/           # Modelo de Driver
│   │   ├── bus.rs      # Abstração de barramento
│   │   ├── class.rs    # Classificação (NIC, Disk)
│   │   ├── device.rs   # Instância de HW físico
│   │   └── driver.rs   # ⚠️ TODO: Driver Trait
│   ├── block/          # Armazenamento
│   │   ├── ahci.rs, nvme.rs, ramdisk.rs
│   ├── input/          # Teclado/Mouse
│   ├── irq/            # Controladores de IRQ
│   ├── net/            # Placas de rede
│   ├── pci/            # PCI Express
│   │   ├── config.rs, pci.rs
│   ├── serial/         # UART para debug
│   ├── timer/          # Fontes de tempo
│   │   ├── hpet.rs, pit.rs, tsc.rs
│   └── video/          # Saída gráfica
│       ├── font.rs, framebuffer.rs
│
├── fs/                 # 📂 Sistema de Arquivos
│   ├── vfs/            # Virtual File System
│   │   ├── dentry.rs   # Cache de diretórios
│   │   ├── file.rs     # Arquivo aberto
│   │   ├── inode.rs    # Metadados
│   │   ├── mount.rs    # Pontos de montagem
│   │   └── path.rs     # Parsing de caminhos
│   ├── devfs/          # /dev/null, /dev/sda
│   ├── initramfs/      # FS temporário em RAM
│   ├── procfs/         # /proc
│   ├── sysfs/          # /sys
│   └── tmpfs/          # Storage volátil
│
├── ipc/                # 📡 Comunicação entre Processos
│   ├── channel/        # Comunicação 1:1
│   ├── futex/          # Fast Userspace Mutex
│   ├── message/        # Envelope de mensagem
│   ├── pipe/           # Fluxo unidirecional
│   ├── port/           # Endpoints de comunicação
│   └── shm/            # Shared Memory
│
├── klib/               # 📚 Biblioteca do Kernel
│   ├── align.rs        # Alinhamento de memória
│   ├── bitmap.rs       # Gerenciamento de bits
│   ├── mem_funcs.rs    # memcpy, memset
│   ├── hash/           # Tabela Hash
│   ├── list/           # Lista duplamente ligada
│   ├── string/         # String sem std
│   └── tree/           # Red-Black Tree
│
├── mm/                 # 🧩 Gerenciamento de Memória
│   ├── addr/           # Wrappers type-safe
│   │   ├── phys.rs, virt.rs, translate.rs
│   ├── alloc/          # Alocadores
│   │   ├── buddy.rs    # Páginas (potências de 2)
│   │   ├── slab.rs     # Objetos pequenos
│   │   ├── bump.rs     # Boot inicial
│   │   └── percpu.rs   # Alocador por CPU
│   ├── cache/          # Page Cache
│   ├── heap/           # GlobalAlloc wrapper
│   ├── ops/            # memset/memcpy seguros
│   ├── pmm/            # Physical Memory Manager
│   │   ├── frame.rs    # Abstração de frame
│   │   ├── zones.rs    # DMA, Normal, HighMem
│   │   ├── bitmap.rs   # Tracking de frames
│   │   └── stats.rs    # Estatísticas
│   ├── types/          # VMO, Pinned
│   └── vmm/            # Virtual Memory Manager
│       ├── mapper.rs   # Page Tables
│       ├── tlb.rs      # TLB management
│       └── vmm.rs      # VMAs por processo
│
├── module/             # 🔒 Sistema de Módulos
│   ├── abi.rs          # Interface estável
│   ├── capability.rs   # Capabilities de módulo
│   ├── loader.rs       # Parser ELF
│   ├── sandbox.rs      # Restrições
│   ├── supervisor.rs   # Ciclo de vida
│   ├── verifier.rs     # Assinatura cripto
│   └── watchdog.rs     # Detecção de travamento
│
├── sched/              # ⚙️ Scheduler
│   ├── context/        # Salvar/Restaurar estado
│   │   └── switch.rs   # Context switch
│   ├── exec/           # Carregadores de executáveis
│   │   ├── elf/        # ELF loader
│   │   ├── interp/     # Scripts
│   │   └── spawn/      # Criação de processo
│   ├── scheduler/      # Algoritmo de decisão
│   │   ├── policy.rs   # Round-Robin/Priority
│   │   ├── runqueue.rs # Fila de prontos
│   │   └── load.rs     # Balanceamento
│   ├── signal/         # Sinais (delivery, handler)
│   ├── task/           # Processo/Thread
│   │   ├── state.rs    # Ready, Running, Blocked
│   │   ├── thread.rs   # TCB
│   │   └── exit.rs     # Cleanup
│   └── wait/           # Wait Queues
│
├── security/           # 🛡️ Segurança
│   ├── audit/          # Log de segurança
│   ├── capability/     # Capability tokens
│   ├── credentials/    # UID, SID, Tokens
│   └── sandbox/        # Namespaces, isolamento
│
├── sync/               # 🔐 Sincronização
│   ├── atomic/         # Operações atômicas
│   ├── condvar/        # Condition Variable
│   ├── mutex/          # Bloqueio com sleep
│   ├── rcu/            # Read-Copy-Update
│   ├── rwlock/         # Reader-Writer Lock
│   ├── semaphore/      # Contagem de recursos
│   └── spinlock/       # Loop ativo
│
├── sys/                # 📋 Definições Compartilhadas
│   ├── elf.rs          # Tipos ELF
│   ├── error.rs        # Códigos de erro
│   └── types.rs        # Tipos comuns
│
└── syscall/            # 🚪 Interface User/Kernel
    ├── abi/            # Convenção de chamada
    │   ├── args.rs, flags.rs, types.rs
    ├── dispatch/       # Tabela de despacho
    ├── event/          # poll()
    ├── fs/             # open, read, write
    ├── handle/         # HandleTable, Rights
    ├── ipc/            # Syscalls de IPC
    ├── memory/         # alloc, map, unmap
    ├── process/        # exit, spawn, wait
    ├── system/         # sysinfo
    ├── time/           # clock_get, sleep
    ├── error.rs        # SysError
    └── numbers.rs      # Constantes IMUTÁVEIS
```

---

## 5. Análise por Módulo

### 📂 `src/arch/` — HAL (Hardware Abstraction Layer)

**Propósito**: Isolar 100% do código específico de CPU. O resto do kernel não sabe que roda em x86_64.

#### Estrutura Atual: ✅ Adequada

| Subpasta | Conteúdo Esperado | Estado |
|----------|-------------------|--------|
| `traits/` | Contratos abstratos (`Cpu`, `Mmu`) | ⚠️ Esquelético |
| `x86_64/` | Implementação concreta | ✅ Funcional |
| `x86_64/acpi/` | Parser de tabelas ACPI | ⚠️ Básico |
| `x86_64/apic/` | LAPIC/IOAPIC | ⚠️ Básico |
| `x86_64/iommu/` | Intel VT-d | ⚠️ Stub |

#### O que DEVE estar em cada arquivo:

| Arquivo | Conteúdo |
|---------|----------|
| `traits/cpu.rs` | `trait Cpu { fn halt(); fn disable_ints(); fn enable_ints(); fn core_id() -> u32; }` |
| `x86_64/cpu.rs` | Impl do trait, leitura de MSRs, CR0/CR3/CR4 |
| `x86_64/gdt.rs` | Segmentos: Kernel Code/Data (Ring 0), User Code/Data (Ring 3), TSS |
| `x86_64/idt.rs` | 256 entradas, handlers para #PF, #GP, #DF, IRQs 32-255 |
| `x86_64/syscall.rs` | Configurar LSTAR, STAR, SFMASK para `syscall` instruction |

---

### 📂 `src/core/` — Núcleo Lógico

**Propósito**: Orquestração agnóstica de hardware. Nunca contém `asm!`.

#### Estrutura Atual: ✅ Excelente

A subdivisão em `boot/`, `object/`, `work/`, `power/`, `smp/`, `time/`, `debug/` demonstra arquitetura orientada a serviços.

#### O que DEVE estar em cada subpasta:

##### `core/boot/`
| Arquivo | Conteúdo |
|---------|----------|
| `entry.rs` | `kernel_main()`: Inicializa subsistemas na ordem `Logger → MM → Sched → Syscall → Init` |
| `handoff.rs` | `BootInfo`: Estrutura idêntica ao bootloader (`#[repr(C)]`) |
| `cmdline.rs` | Parser de args: `debug=on`, `root=/dev/nvme0`, `console=serial` |
| `panic.rs` | `#[panic_handler]`: Dump de estado, halt |
| `initcall.rs` | Registrar funções que rodam no boot (estilo `module_init`) |

##### `core/object/`
| Arquivo | Conteúdo |
|---------|----------|
| `kobject.rs` | `trait KernelObject { fn type_id(); fn ref_count(); }` |
| `handle.rs` | `Handle`: u32 opaco para userspace, indexa `CSpace` |
| `rights.rs` | `bitflags! { READ, WRITE, EXECUTE, TRANSFER, DUPLICATE }` |
| `dispatcher.rs` | `dispatch(handle) → &dyn KernelObject` |
| `refcount.rs` | Contagem atômica, `Arc`-like para kernel |

##### `core/smp/`
| Arquivo | Conteúdo |
|---------|----------|
| `bringup.rs` | Wake APs via SIPI (Startup IPI) |
| `ipi.rs` | `send_ipi(target_cpu, vector)` |
| `percpu.rs` | `#[percpu] static CURRENT_TASK: Option<&Task>` |
| `topology.rs` | Descobrir cores, sockets, HT via CPUID/MADT |

---

### 📂 `src/drivers/` — Drivers e Barramentos

**Propósito**: Conectar hardware aos subsistemas. Interfaces aqui, implementações complexas em módulos.

#### ⚠️ Problema Crítico: `driver.rs` está VAZIO

```rust
// drivers/base/driver.rs
//! TODO: Driver trait
```

**Isso é bloqueante.** Sem o contrato `Driver`, não existe definição formal de como drivers interagem com o kernel.

#### O que DEVE estar em `driver.rs`:

```rust
/// Trait que todo driver DEVE implementar
pub trait Driver: Send + Sync {
    /// Nome único do driver
    fn name(&self) -> &'static str;
    
    /// Tipo de dispositivo que o driver gerencia
    fn device_type(&self) -> DeviceType;
    
    /// Chamado quando dispositivo compatível é detectado
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado quando dispositivo é removido
    fn remove(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado durante suspend (S3)
    fn suspend(&self, dev: &mut Device) -> Result<(), DriverError> {
        Ok(()) // Default: no-op
    }
    
    /// Chamado durante resume
    fn resume(&self, dev: &mut Device) -> Result<(), DriverError> {
        Ok(()) // Default: no-op
    }
}

/// Tipos de dispositivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Block,      // Armazenamento
    Char,       // Serial, tty
    Network,    // NICs
    Input,      // Teclado, mouse
    Display,    // GPU, framebuffer
    Bus,        // PCI, USB controller
    Unknown,
}
```

---

### 📂 `src/mm/` — Gerenciamento de Memória

**Propósito**: Gerenciar RAM física e virtual.

#### Estrutura Atual: ✅ Bem estruturada

A divisão em `pmm/`, `vmm/`, `alloc/`, `heap/` é excelente.

#### O que DEVE estar em cada subpasta:

##### `mm/pmm/` (Physical Memory Manager)
| Arquivo | Conteúdo |
|---------|----------|
| `frame.rs` | `PhysFrame`: Abstração de frame de 4KB |
| `zones.rs` | `Zone`: DMA (<16MB), Normal, HighMem (>4GB) |
| `bitmap.rs` | Bitmap de frames livres/usados |
| `stats.rs` | `PmmStats { total, used, free, reserved }` |

##### `mm/vmm/` (Virtual Memory Manager)
| Arquivo | Conteúdo |
|---------|----------|
| `mapper.rs` | `map_page(virt, phys, flags)`, `unmap_page(virt)` |
| `tlb.rs` | `invlpg(virt)`, `flush_all()`, TLB shootdown |
| `vmm.rs` | `AddressSpace`: Lista de VMAs por processo |

##### `mm/alloc/`
| Arquivo | Conteúdo |
|---------|----------|
| `buddy.rs` | Buddy Allocator para páginas (potências de 2) |
| `slab.rs` | Slab Allocator para objetos pequenos (Task, Inode) |
| `bump.rs` | Bump allocator para early-boot |
| `percpu.rs` | Alocador local por CPU (reduz contention) |

---

### 📂 `src/sched/` — Scheduler

**Propósito**: Decidir qual tarefa roda na CPU.

#### ⚠️ Problemas Identificados

1. **Global Lock**: `SCHEDULER` usa Mutex único → gargalo em SMP
2. **Falta FPU State**: Contexto não salva SSE/AVX → corrompe apps
3. **Falta Per-CPU Runqueues**: Essencial para escalabilidade

#### O que DEVE estar em `context/`:

```rust
/// Contexto de CPU completo
#[repr(C)]
pub struct CpuContext {
    // Registradores de propósito geral
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    
    // Registradores de segmento
    pub cs: u64, pub ss: u64, pub ds: u64, pub es: u64,
    pub fs: u64, pub gs: u64,
    
    // Estado de controle
    pub rip: u64,
    pub rflags: u64,
    
    // FPU/SSE/AVX (CRÍTICO!)
    pub fxsave_area: [u8; 512], // FXSAVE/FXRSTOR area
}
```

---

### 📂 `src/module/` — Sistema de Módulos

**Propósito**: Carregar código dinâmico (drivers) de forma segura.

#### Estrutura Atual: ✅ Excelente design

Esta pasta **valida a arquitetura**. Módulos são tratados como código não-confiável.

#### O que DEVE estar em cada arquivo:

| Arquivo | Conteúdo |
|---------|----------|
| `abi.rs` | Interface binária estável: `ModuleAbi { version, init, cleanup, name, caps_requested }` |
| `loader.rs` | Parser de ELF relocável, resolve símbolos |
| `verifier.rs` | Verificação de assinatura Ed25519/RSA-4096 |
| `sandbox.rs` | Configura restrições: sem acesso a page tables, sem DMA direto |
| `supervisor.rs` | Gerencia ciclo de vida, registra módulo ativo |
| `watchdog.rs` | Detecta módulos travados (timeout de healthcheck) |
| `capability.rs` | `ModuleCapType { DmaAccess, IrqHandler, MmioRegion, ... }` |

---

### 📂 `src/syscall/` — Interface User/Kernel

**Propósito**: Única porta de entrada. Tudo passa por aqui.

#### Estrutura Atual: ✅ Bem organizada

Dispatch table-based é a escolha correta para O(1) lookup.

#### O que DEVE estar em cada subpasta:

##### `syscall/dispatch/`
| Arquivo | Conteúdo |
|---------|----------|
| `table.rs` | `static SYSCALL_TABLE: [fn; 256]` — Tabela de handlers |
| `mod.rs` | `syscall_dispatcher(num, args) → Result` |

##### `syscall/abi/`
| Arquivo | Conteúdo |
|---------|----------|
| `args.rs` | `SyscallArgs { a0..a5: u64 }` — Argumentos raw |
| `flags.rs` | Flags comuns (O_RDONLY, MAP_ANONYMOUS, etc.) |
| `version.rs` | `ABI_VERSION = 1` — Versionamento de ABI |

---

## 6. Lacunas Críticas Identificadas

### 🔴 Prioridade Crítica (Bloqueia funcionamento)

| ID | Lacuna | Impacto | Localização |
|----|--------|---------|-------------|
| **G1** | `Driver` trait vazio | Drivers não podem existir | `drivers/base/driver.rs` |
| **G2** | Contexto FPU ausente | Corrompe apps com float | `sched/context/` |
| **G3** | CSpace não implementado | Capabilities são placeholders | `security/` |
| **G4** | IPC não integrado ao scheduler | Busy-wait desperdiça CPU | `ipc/`, `sched/wait/` |

### 🟡 Prioridade Alta (Limita funcionalidade)

| ID | Lacuna | Impacto | Localização |
|----|--------|---------|-------------|
| **G5** | Global lock no scheduler | Gargalo em SMP | `sched/scheduler/` |
| **G6** | Alocadores per-CPU ausentes | Contention de heap | `mm/alloc/percpu.rs` |
| **G7** | Revogação de capabilities | Vazamento de permissões | `security/capability/` |
| **G8** | Zero-copy IPC | Overhead de memcpy | `ipc/shm/` |

### 🟢 Prioridade Média (Melhoria de qualidade)

| ID | Lacuna | Impacto | Localização |
|----|--------|---------|-------------|
| **G9** | Async drivers | Modelo callback é antigo | `drivers/base/` |
| **G10** | KASLR | Segurança reduzida | `mm/vmm/` |
| **G11** | Watchdog de kernel | Deadlocks não detectados | `core/debug/` |

---

## 7. Plano de Implementação por Fases

### 🏁 Fase 0: Fundação (Atual → Boot Estável)

**Objetivo**: Kernel boota e executa init process com userspace funcional.

| Tarefa | Arquivos | Estimativa |
|--------|----------|------------|
| Implementar `Driver` trait | `drivers/base/driver.rs` | 4h |
| Adicionar contexto FPU | `sched/context/switch.rs` | 8h |
| Integrar IPC com wait queues | `ipc/port.rs`, `sched/wait/` | 6h |
| Syscalls básicos funcionais | `syscall/fs/`, `process/` | 8h |

**Critério de Sucesso**: `/system/core/init` executa em Ring 3.

---

### 🔧 Fase 1: Segurança Real

**Objetivo**: Capabilities funcionam de verdade.

| Tarefa | Arquivos | Estimativa |
|--------|----------|------------|
| Implementar CSpace/CNode | `security/capability/` | 16h |
| Adicionar revogação | `security/capability/` | 8h |
| Handles verificados em syscalls | `syscall/handle/` | 8h |
| Audit logging | `security/audit/` | 4h |

**Critério de Sucesso**: Processo não pode acessar recursos sem handle válido.

---

### ⚡ Fase 2: Performance

**Objetivo**: Kernel escala em SMP.

| Tarefa | Arquivos | Estimativa |
|--------|----------|------------|
| Per-CPU runqueues | `sched/scheduler/runqueue.rs` | 12h |
| Alocadores per-CPU | `mm/alloc/percpu.rs` | 8h |
| TLB shootdown otimizado | `mm/vmm/tlb.rs` | 6h |
| Zero-copy IPC | `ipc/shm/` | 8h |

**Critério de Sucesso**: Benchmark mostra scaling linear com cores.

---

### 🧩 Fase 3: Módulos Externos

**Objetivo**: Drivers podem ser carregados dinamicamente.

| Tarefa | Arquivos | Estimativa |
|--------|----------|------------|
| ELF loader para módulos | `module/loader.rs` | 12h |
| Verificação de assinatura | `module/verifier.rs` | 8h |
| Sandbox com IOMMU | `module/sandbox.rs` | 16h |
| Watchdog de módulos | `module/watchdog.rs` | 4h |

**Critério de Sucesso**: `nvidia.ko` carrega e gerencia GPU sem acesso direto ao kernel.

---

## 8. Requisitos Detalhados por Arquivo

### 📋 Arquivos Críticos com Especificação

#### `drivers/base/driver.rs`

```rust
//! Driver Interface Contract
//!
//! Define o que significa ser um driver no Redstone OS.

/// Erro de driver
#[derive(Debug)]
pub enum DriverError {
    NotSupported,
    InitFailed,
    BusError,
    ResourceBusy,
}

/// Tipo de dispositivo
#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Block,
    Char,
    Network,
    Input,
    Display,
    Bus,
}

/// Trait obrigatório para todos os drivers
pub trait Driver: Send + Sync + 'static {
    /// Nome do driver (ex: "ahci", "nvme", "e1000")
    fn name(&self) -> &'static str;
    
    /// Tipo de dispositivo
    fn device_type(&self) -> DeviceType;
    
    /// Chamado quando dispositivo é detectado
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado quando dispositivo é removido
    fn remove(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// (Opcional) Suspend
    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> { Ok(()) }
    
    /// (Opcional) Resume
    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> { Ok(()) }
}
```

#### `sched/context/switch.rs`

```rust
//! Context Switch Implementation
//!
//! Salva e restaura estado completo de CPU, incluindo FPU/SSE.

/// Área de salvamento FPU (512 bytes, alinhado 16)
#[repr(C, align(16))]
pub struct FxSaveArea([u8; 512]);

/// Contexto completo de CPU
#[repr(C)]
pub struct CpuContext {
    // GPRs (ordenados para switch.s)
    pub rsp: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    
    // FPU/SSE state
    pub fpu_state: FxSaveArea,
}

impl CpuContext {
    /// Salva estado FPU atual
    pub fn save_fpu(&mut self) {
        unsafe {
            core::arch::asm!("fxsave [{}]", in(reg) &mut self.fpu_state);
        }
    }
    
    /// Restaura estado FPU
    pub fn restore_fpu(&self) {
        unsafe {
            core::arch::asm!("fxrstor [{}]", in(reg) &self.fpu_state);
        }
    }
}
```

#### `security/capability/mod.rs`

```rust
//! Capability-Based Security
//!
//! Implementação de capabilities estilo seL4/Zircon.

use bitflags::bitflags;

bitflags! {
    /// Direitos que uma capability pode ter
    #[derive(Debug, Clone, Copy)]
    pub struct CapRights: u32 {
        const READ      = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const DUPLICATE = 0b0000_1000;
        const TRANSFER  = 0b0001_0000;
        const GRANT     = 0b0010_0000;
        const REVOKE    = 0b0100_0000;
    }
}

/// Tipo de objeto que a capability referencia
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CapType {
    Null,           // Slot vazio
    Memory,         // VMO
    Port,           // IPC Port
    Thread,         // Thread handle
    Process,        // Process handle
    Irq,            // IRQ handler
    Mmio,           // MMIO region
    CNode,          // Container de capabilities
}

/// Uma capability é um token unforgeable
#[derive(Debug)]
pub struct Capability {
    /// Tipo do objeto
    pub cap_type: CapType,
    /// Direitos associados
    pub rights: CapRights,
    /// Referência ao objeto real (opaco)
    pub object_ref: u64,
    /// Badge para identificação (usado em IPC)
    pub badge: u64,
}

/// CSpace: Tabela de capabilities por processo
pub struct CSpace {
    /// Array de slots (simplificado; produção usaria radix tree)
    slots: [Option<Capability>; 1024],
    /// Próximo slot livre
    next_free: usize,
}

impl CSpace {
    /// Aloca novo slot e insere capability
    pub fn insert(&mut self, cap: Capability) -> Option<CapHandle> {
        if self.next_free >= self.slots.len() {
            return None;
        }
        let handle = CapHandle(self.next_free as u32);
        self.slots[self.next_free] = Some(cap);
        self.next_free += 1;
        Some(handle)
    }
    
    /// Busca capability por handle
    pub fn lookup(&self, handle: CapHandle) -> Option<&Capability> {
        self.slots.get(handle.0 as usize)?.as_ref()
    }
    
    /// Revoga capability
    pub fn revoke(&mut self, handle: CapHandle) {
        if let Some(slot) = self.slots.get_mut(handle.0 as usize) {
            *slot = None;
        }
    }
}

/// Handle opaco para userspace
#[derive(Debug, Clone, Copy)]
pub struct CapHandle(pub u32);
```

---

## 9. Critérios de Aceitação

### ✅ Checklist de Qualidade

#### Código
- [ ] Zero `unwrap()` ou `expect()` fora de testes
- [ ] Todo `unsafe` tem comentário `// SAFETY:`
- [ ] Nenhum `f32`/`f64` no kernel
- [ ] Nenhuma dependência externa em `Cargo.toml`
- [ ] CI passa com `cargo clippy -- -D warnings`

#### Arquitetura
- [ ] Nenhum `asm!` fora de `src/arch/`
- [ ] `core/` nunca importa de `arch/x86_64/` diretamente (usa traits)
- [ ] Syscalls validam handles antes de usar
- [ ] Modules não podem acessar `KERNEL_*` symbols

#### Funcionalidade
- [ ] Boot até init em <500ms (debug), <100ms (release)
- [ ] Processos de usuário rodam isolados
- [ ] Contexto FPU preservado entre context switches
- [ ] IPC funciona com blocking (não busy-wait)

---

## 10. Glossário Técnico

| Termo | Definição |
|-------|-----------|
| **ABI** | Application Binary Interface - contrato binário entre kernel e userspace |
| **BSS** | Block Started by Symbol - seção de variáveis não inicializadas |
| **Capability** | Token unforgeable que representa permissão de acesso |
| **CSpace** | Capability Space - tabela de capabilities por processo |
| **FPU** | Floating Point Unit - processador de ponto flutuante |
| **GDT** | Global Descriptor Table - segmentos de memória x86 |
| **HAL** | Hardware Abstraction Layer |
| **IDT** | Interrupt Descriptor Table |
| **IOMMU** | I/O Memory Management Unit - proteção de DMA |
| **IPC** | Inter-Process Communication |
| **LAPIC** | Local Advanced PIC - timer e IPI por core |
| **MSR** | Model Specific Register |
| **PMM** | Physical Memory Manager |
| **SMP** | Symmetric Multiprocessing |
| **TLB** | Translation Lookaside Buffer - cache de page tables |
| **VMM** | Virtual Memory Manager |
| **VMO** | Virtual Memory Object |

---

> **Última atualização**: Dezembro 2024  
> **Próxima revisão**: Após conclusão da Fase 0
