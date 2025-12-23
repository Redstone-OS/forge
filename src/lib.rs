// (FASE2) src/lib.rs
//! Forge Kernel Library.
//!
//! Ponto central de exportação dos módulos do Kernel.
//! Define a estrutura hierárquica do sistema operacional.

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

// Habilitar alocação dinâmica (necessário para Vec/Box/Arc)
extern crate alloc;

// --- Módulos de Baixo Nível (Hardware) ---
pub mod arch; // HAL (CPU, GDT, IDT)
pub mod drivers; // Drivers Específicos (Serial, Video, Timer)

// --- Módulos Centrais (Lógica do Kernel) ---
pub mod core; // Inicialização, Panic, Handoff
pub mod klib; // Utilitários Internos (Bitmaps, Math)
pub mod mm; // Gerenciamento de Memória (PMM, VMM, Heap)
pub mod sync;
pub mod sys; // Definições de Sistema (ABI, Erros) // Primitivas de Sincronização (Mutex)

// --- Subsistemas Avançados ---
pub mod fs; // Sistema de Arquivos Virtual (VFS)
pub mod ipc; // Comunicação entre Processos
pub mod sched; // Scheduler e Tarefas
pub mod security; // Capabilities
pub mod syscall; // Interface com Userspace

// Re-exportar BootInfo para acesso fácil no binário
pub use crate::core::handoff::BootInfo;
