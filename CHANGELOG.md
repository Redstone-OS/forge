# 📋 Changelog

Todas as mudanças notáveis do projeto Forge Kernel serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [0.2.0] - 2026-01-04

### 🏗️ Refatoração Arquitetural

Esta versão marca uma **redesign completa** da base estrutural do projeto, consolidando os módulos em uma arquitetura em camadas bem definida.

### ✨ Adicionado

- **Documentação Completa**: 13 arquivos de documentação técnica em `doc/`
  - `ARCH.md` - Hardware Abstraction Layer
  - `CORE.md` - Core subsystem (boot, smp, time, work, debug)
  - `DRIVERS.md` - Redstone Drive System (RDS)
  - `FS.md` - Sistema de arquivos completo
  - `KLIB.md` - Kernel library
  - `SECURITY.md` - Modelo OCAP
  - `SYNC.md` - Primitivas de sincronização
  - `SYS.md` - Tipos e definições do sistema
  - `SYSCALL.md` - Interface de syscalls
  - `IPC.md` - Inter-process communication
  - `MM.md` - Gerenciamento de memória
  - `SCHED.md` - Scheduler
  - `MODULE.md` - Sistema de módulos

- **Redstone Drive System (RDS)**: Framework unificado de drivers
  - `drivers/base/` - Infraestrutura central com DriverManager
  - `drivers/bus/` - Suporte a PCI, USB, VirtIO, ACPI, ISA
  - Sistema de recuperação automática de falhas
  - DMA Pool centralizado
  - Driver Zone (memória isolada)
  - Hot-reload de drivers
  - Fallback progressivo

- **Core Subsystem Reorganizado**
  - `core/boot/` - Entry, panic, initcalls, command line
  - `core/debug/` - Logging, stats, oops, tracing
  - `core/smp/` - Multiprocessamento, IPI, per-CPU
  - `core/time/` - Clocks, timers, jiffies
  - `core/work/` - Work queues diferidas
  - `core/power/` - Shutdown, reboot
  - `core/object/` - Handles do kernel

- **Sistema de Segurança OCAP**
  - `security/capability/` - Capabilities, CSpace, revogação
  - `security/audit/` - Eventos e logging de segurança
  - `security/sandbox/` - Namespaces e containers

- **Sync Primitivas Completas**
  - `sync/spinlock/` - Interrupt-safe
  - `sync/mutex/` - Sleep-capable (spin-wait atual)
  - `sync/rwlock/` - Read-Write lock
  - `sync/semaphore/` - Resource counting
  - `sync/condvar/` - Condition variables
  - `sync/rcu/` - Read-Copy-Update
  - `sync/atomic/` - Wrappers atômicos

- **Kernel Library (klib)**
  - `klib/primitives/` - align, bits, mem
  - `klib/collections/` - bitmap, intrusive list, ring buffer
  - `klib/hash/` - FNV hasher
  - `klib/cstr/` - C-string utilities
  - `klib/bitflags.rs` - Macro interna

- **Sys Types**
  - NewTypes seguros: `Pid`, `Tid`, `Uid`, `Gid`
  - `KernelError` enum com 16 variantes
  - ELF64 parser para loader

- **Driver Categories**
  - `drivers/storage/` - AHCI, NVMe, VirtIO-Blk, Ramdisk, ATA
  - `drivers/network/` - Ethernet (Intel, Realtek), VirtIO-Net
  - `drivers/display/` - Framebuffer, GPU, VirtIO-GPU
  - `drivers/input/` - HID, PS/2, VirtIO-Input, Touch
  - `drivers/sound/` - HD Audio, AC97, VirtIO-Sound
  - `drivers/system/` - Timer, INT controller, DMA, power
  - `drivers/comm/` - Serial, I2C, SPI

### 🔧 Modificado

- **Arquitetura em Camadas**: Reorganização de todos os módulos em 5 layers claras
  - L0: Hardware (arch, drivers)
  - L1: Primitivas (klib, sync, sys)
  - L2: Core (core, mm, sched)
  - L3: Serviços (ipc, fs, security, module)
  - L4: Interface (syscall)

- **lib.rs**: Completamente reescrito com documentação da arquitetura

- **Sistema de Logging**: Macros `kinfo!`, `kwarn!`, `kerror!`, `kdebug!`, `ktrace!`

- **Boot Sequence**: Ordem de inicialização bem definida e documentada

### 🗑️ Removido

- Código legado e duplicado
- Dependências externas (política de zero dependências)
- Implementações inconsistentes

---

## [0.1.5] - 2025-12-XX

### 📚 Documentação Inicial

Esta versão focou na criação da documentação básica do projeto.

### ✨ Adicionado

- Estrutura inicial de documentação
- README.md com visão geral
- Descrição básica dos módulos

### 🔧 Modificado

- Organização inicial de diretórios
- Melhorias no sistema de build

---

## [0.1.0] - 2025-XX-XX

### 🎉 Release Inicial

Primeira versão funcional do Forge Kernel.

### ✨ Adicionado

- **Boot**: Inicialização via Limine bootloader
- **arch/x86_64**: GDT, IDT, APIC básico
- **mm**: PMM e VMM funcionais
- **sched**: Scheduler round-robin
- **fs**: VFS básico, FAT read-only, InitRAMFS
- **syscall**: Interface básica de syscalls
- **drivers**: Serial, timer PIT, framebuffer

### 🧪 Testes

- Boot em QEMU funcional
- Carregamento de serviços do userspace
- Execução do supervisor

---

## Legenda

- ✨ **Adicionado**: Novas funcionalidades
- 🔧 **Modificado**: Mudanças em funcionalidades existentes
- 🗑️ **Removido**: Funcionalidades removidas
- 🐛 **Corrigido**: Correções de bugs
- 🔒 **Segurança**: Correções de vulnerabilidades
- 🧪 **Testes**: Adições ou mudanças em testes
- 📚 **Documentação**: Atualizações na documentação
- 🏗️ **Refatoração**: Mudanças que não afetam comportamento

---

<div align="center">

*Forge Kernel — RedstoneOS*

</div>
