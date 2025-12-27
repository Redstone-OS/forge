//! # Redstone OS Kernel (Forge)
//!
//! O núcleo do sistema operacional, responsável por orquestrar hardware e software.
//!
//! ## 🏗️ Arquitetura: Micro-Modular Pragmática
//!
//! O `forge` não é um kernel monolítico convencional (Linux), nem um microkernel acadêmico (Minix).
//! Adotamos um meio-termo pragmático focado em:
//! - **Isolamento de Falhas:** Drivers e Serviços rodam isolados (idealmente em userspace ou ring 1).
//! - **Capability-Based Security:** Permissões são tokens, não listas de acesso (ACLs). Zero Trust interno.
//! - **Imutabilidade:** O kernel assume que o sistema de arquivos base é imutável.
//!
//! ## 📦 Estrutura de Módulos (Map)
//!
//! ### Hardware Abstraction Layer (HAL)
//! - [`arch`]: Traduz conceitos abstratos (interrupção, paginação) para o dialeto da CPU (x86_64).
//! - [`drivers`]: Implementações específicas de dispositivos (Serial, Video).
//!
//! ### Core Subsystems
//! - [`mm`]: **Memory Manager**. PMM (Físico) -> VMM (Virtual) -> Heap (Kernel Objects).
//! - [`sched`]: **Scheduler**. Multitarefa preemptiva, threads e contextos.
//! - [`ipc`]: **Inter-Process Communication**. Portas e mensagens. O "barramento" do OS.
//! - [`security`]: **Capabilities**. A autoridade que valida quem pode fazer o quê.
//!
//! ### System Interfaces
//! - [`syscall`]: **API do Userspace**. A fronteira de ataque. Onde o Ring 3 pede coisas ao Ring 0.
//! - [`fs`]: **Virtual File System**. Abstração unificada de armazenamento.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Inicialização Frágil:** O fluxo de `_start` até `init` depende de uma ordem rígida de inicialização de subsistemas (Logger -> MM -> Sched). Erros aqui causam Boot Loop ou Triple Fault.
//! - **Driver Model:** Atualmente os drivers (ex: Serial) estão linkados estaticamente no binário do kernel. Isso é "Monolítico". O objetivo futuro é movê-los para módulos carregáveis ou processos separados.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Architecture)** Definir interface estável para **Drivers Assíncronos** (baseado em `Future`).
//! - [ ] **TODO: (Security)** Implementar **Kernel Address Space Layout Randomization (KASLR)**. O kernel carrega sempre no mesmo endereço físico/virtual hoje.
//! - [ ] **TODO: (Reliability)** Criar um **Watchdog de Kernel** que detecte deadlocks em spinlocks e cause um panic controlado.
//!
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(asm_const)]

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
