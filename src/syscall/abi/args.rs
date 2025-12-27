//! # Syscall Arguments
//!
//! Extração de argumentos do contexto de interrupção/syscall.

use crate::arch::x86_64::idt::ContextFrame;

/// Máximo de argumentos suportados
pub const MAX_ARGS: usize = 6;

/// Argumentos de syscall extraídos do contexto
///
/// Convenção de registradores (x86_64):
/// - RAX: número da syscall
/// - RDI: arg1
/// - RSI: arg2
/// - RDX: arg3
/// - R10: arg4 (RCX é destruído por syscall)
/// - R8:  arg5
/// - R9:  arg6
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    pub num: usize,
    pub arg1: usize,
    pub arg2: usize,
    pub arg3: usize,
    pub arg4: usize,
    pub arg5: usize,
    pub arg6: usize,
}

impl SyscallArgs {
    /// Extrai argumentos do ContextFrame
    pub fn from_context(ctx: &ContextFrame) -> Self {
        Self {
            num: ctx.rax as usize,
            arg1: ctx.rdi as usize,
            arg2: ctx.rsi as usize,
            arg3: ctx.rdx as usize,
            arg4: ctx.r10 as usize,
            arg5: ctx.r8 as usize,
            arg6: ctx.r9 as usize,
        }
    }

    /// Argumentos vazios (para testes)
    pub const fn empty() -> Self {
        Self {
            num: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            arg6: 0,
        }
    }
}
