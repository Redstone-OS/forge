//! # Forge — Redstone OS Kernel
//!
//! Kernel moderno escrito em Rust, focado em segurança e modularidade.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    USERSPACE (Ring 3)                       │
//! └─────────────────────────────────────────────────────────────┘
//!                           ↑ syscall ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │  syscall  │ Única porta de entrada. Valida tudo.            │
//! └─────────────────────────────────────────────────────────────┘
//!                           ↑ handles ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │  core     │ Orquestração: boot, objects, time, work, smp    │
//! │  sched    │ Scheduler: tasks, context switch, runqueues     │
//! │  mm       │ Memória: PMM, VMM, heap, alocadores             │
//! │  ipc      │ Comunicação: ports, channels, shared memory     │
//! │  fs       │ Filesystem: VFS, devfs, tmpfs                   │
//! │  security │ Capabilities: tokens, CSpace, audit             │
//! └─────────────────────────────────────────────────────────────┘
//!                           ↑ traits ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │  arch     │ HAL: CPU, GDT, IDT, APIC, paginação             │
//! │  drivers  │ Hardware: serial, video, timer, PCI             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Princípios
//!
//! - **Capability-Based Security**: Acesso via tokens, não identidade
//! - **Zero Trust**: Módulos são supervisionados, mesmo em Ring 0
//! - **No Legacy**: Sem compatibilidade com padrões antigos

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(raw_ref_op)]
#![feature(panic_info_message)]

extern crate alloc;

// =============================================================================
// LAYER 0: HARDWARE ABSTRACTION
// =============================================================================

/// Hardware Abstraction Layer - isola código específico de CPU
pub mod arch;

/// Drivers de hardware (serial, video, timer, PCI)
pub mod drivers;

// =============================================================================
// LAYER 1: KERNEL PRIMITIVES
// =============================================================================

/// Biblioteca do kernel (bitmap, align, mem_funcs)
pub mod klib;

/// Primitivas de sincronização (spinlock, mutex, rwlock)
pub mod sync;

/// Tipos e definições do sistema
pub mod sys;

// =============================================================================
// LAYER 2: CORE SUBSYSTEMS
// =============================================================================

/// Núcleo do kernel (boot, objects, time, smp, work)
pub mod core;

/// Gerenciamento de memória (PMM, VMM, heap)
pub mod mm;

/// Scheduler e gerenciamento de tarefas
pub mod sched;

// =============================================================================
// LAYER 3: KERNEL SERVICES
// =============================================================================

/// Comunicação entre processos
pub mod ipc;

/// Sistema de arquivos virtual
pub mod fs;

/// Subsistema de segurança (capabilities)
pub mod security;

/// Sistema de módulos carregáveis
pub mod module;

// =============================================================================
// LAYER 4: SYSTEM INTERFACE
// =============================================================================

/// Interface de syscalls (fronteira user/kernel)
pub mod syscall;

// =============================================================================
// PUBLIC API
// =============================================================================

pub use core::boot::BootInfo;
