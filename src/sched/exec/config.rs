//! # Configuração do Módulo Exec
//!
//! Constantes centralizadas para criação de processos e execução.

// =============================================================================
// LAYOUT DE MEMÓRIA - USER SPACE
// =============================================================================

/// Topo da stack de usuário (final da metade inferior canônica x86_64)
///
/// A stack cresce para baixo a partir deste endereço.
/// Valor: 0x7FFF_FFFF_F000 (logo abaixo do hole canônico)
pub const USER_STACK_TOP: u64 = 0x7FFF_FFFF_F000;

/// Base do heap de usuário
///
/// O heap cresce para cima a partir deste endereço via brk/mmap.
pub const USER_HEAP_BASE: u64 = 0x0000_4000_0000_0000;

/// Endereço mínimo para código de usuário
///
/// Segmentos ELF são carregados acima deste endereço.
pub const USER_CODE_MIN: u64 = 0x0000_0000_0040_0000;

// =============================================================================
// LAYOUT DE MEMÓRIA - KERNEL SPACE
// =============================================================================

/// Base para stacks de kernel de processos
///
/// Cada processo recebe uma stack de kernel em:
/// KERNEL_STACK_BASE + (PID * KERNEL_STACK_SIZE)
pub const KERNEL_STACK_BASE: u64 = 0xFFFF_9100_0000_0000;

// Importar tamanhos do config do scheduler
pub use crate::sched::config::{KERNEL_STACK_SIZE, USER_STACK_SIZE};

// =============================================================================
// SELETORES DE SEGMENTO - x86_64
// =============================================================================

/// Seletor de código para userspace (Ring 3)
///
/// GDT Index 4, RPL 3 = (4 << 3) | 3 = 0x23
pub const USER_CODE_SELECTOR: u64 = 0x23;

/// Seletor de dados para userspace (Ring 3)
///
/// GDT Index 3, RPL 3 = (3 << 3) | 3 = 0x1B
pub const USER_DATA_SELECTOR: u64 = 0x1B;

/// RFLAGS inicial para processos de usuário
///
/// IF=1 (Interrupt Flag), bit reservado=1
pub const USER_RFLAGS: u64 = 0x202;

// =============================================================================
// CONTEXT SWITCH
// =============================================================================

/// Espaço reservado na stack para context switch
///
/// O context_switch consome este espaço antes de saltar.
pub const SWITCH_RESERVE: u64 = 8;
