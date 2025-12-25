//! # Syscall ABI (x86_64)
//!
//! Este módulo define o contrato binário (Application Binary Interface) para chamadas de sistema.
//! A estabilidade deste contrato é o que permite que aplicações continuem rodando mesmo após
//! atualizações do kernel.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Standardization:** Define como registradores são mapeados para argumentos.
//! - **Data Structures:** Define o layout de memória de estruturas compartilhadas (`IoVec`, `TimeSpec`).
//!
//! ## 🏗️ Arquitetura: System V AMD64 (Modified)
//! Adotamos a convenção de passagem de parâmetros da System V (Linux), mas com especificidades:
//! - **Instruction:** `int 0x80` (Legado/Compat) e `syscall` (Moderno/Rápido).
//! - **Clobbers:** RCX e R11 são destruídos pela instrução `syscall`. O kernel preserva os demais (RBX, RBP, R12-R15).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Compatibilidade Mecânica:** Usar a mesma ordem de registradores do Linux (`rdi, rsi, rdx...`) facilita o port de compiladores (LLVM/Rustc) e libc.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Lack of Alignment Check:** `SyscallArgs` assume que o `ContextFrame` está alinhado. Se o trampoline falhar, o cast é UB.
//! - **Manual Padding:** `TimeSpec` tem padding manual (`_pad`). Seria melhor usar `#[repr(align(16))]` ou similiar para garantir alinhamento explícito sem campos fantasmas.
//! - **IO VEC Validity:** `IoVec::is_valid` é muito simplório. Ele checa `null`, mas não checa se o range `base..base+len` está totalmente dentro do espaço de usuário (canonica address check).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Security)** Implementar **Pointer Sanitizer**.
//!   - *Meta:* `IoVec` deve validar se o range de memória toca em endereços de kernel (`> 0x0000_7FFF_FFFF_FFFF`).
//! - [ ] **TODO: (Performance)** Migrar exclusivamente para instrução **`syscall`**.
//!   - *Motivo:* `int 0x80` é muito mais lenta devido ao overhead de tratamento de interrupção (hardware context save).
//!
//! --------------------------------------------------------------------------------
//!
//! Define a convenção de chamada e estruturas para syscalls.
//!
//! # Convenção de Registradores
//!
//! | Registrador | Uso                    |
//! |-------------|------------------------|
//! | RAX         | Número da syscall      |
//! | RDI         | Argumento 1            |
//! | RSI         | Argumento 2            |
//! | RDX         | Argumento 3            |
//! | R10         | Argumento 4            |
//! | R8          | Argumento 5            |
//! | R9          | Argumento 6            |
//! | RAX         | Retorno (valor ou -errno) |
//!
//! # Método de Invocação
//!
//! Atualmente: `int 0x80` via IDT
//! Futuro: `syscall` instrução (mais rápido)

/// Máximo de argumentos suportados por syscall
pub const MAX_SYSCALL_ARGS: usize = 6;

/// Vetor de interrupção para syscalls
pub const SYSCALL_VECTOR: u8 = 0x80;

/// Estrutura com argumentos de syscall (extraídos do contexto)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    /// Número da syscall (RAX)
    pub num: usize,
    /// Argumento 1 (RDI)
    pub arg1: usize,
    /// Argumento 2 (RSI)
    pub arg2: usize,
    /// Argumento 3 (RDX)
    pub arg3: usize,
    /// Argumento 4 (R10)
    pub arg4: usize,
    /// Argumento 5 (R8)
    pub arg5: usize,
    /// Argumento 6 (R9)
    pub arg6: usize,
}

impl SyscallArgs {
    /// Extrai argumentos do ContextFrame da interrupção
    pub fn from_context(ctx: &crate::arch::x86_64::idt::ContextFrame) -> Self {
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
}

/// Estrutura para IO vetorizado (similar a iovec)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IoVec {
    /// Ponteiro para o buffer
    pub base: *mut u8,
    /// Tamanho do buffer em bytes
    pub len: usize,
}

impl IoVec {
    /// Cria IoVec vazio
    pub const fn empty() -> Self {
        Self {
            base: core::ptr::null_mut(),
            len: 0,
        }
    }

    /// Verifica se o IoVec é válido (não nulo e tamanho > 0)
    pub fn is_valid(&self) -> bool {
        !self.base.is_null() && self.len > 0
    }
}

/// Flags para operações de IO
pub mod io_flags {
    /// Operação não deve bloquear
    pub const NONBLOCK: u32 = 1 << 0;
    /// Append ao final (escrita)
    pub const APPEND: u32 = 1 << 1;
    /// Operação deve ser síncrona (flush)
    pub const SYNC: u32 = 1 << 2;
}

/// Flags para mapeamento de memória
pub mod map_flags {
    /// Memória legível
    pub const READ: u32 = 1 << 0;
    /// Memória gravável
    pub const WRITE: u32 = 1 << 1;
    /// Memória executável
    pub const EXEC: u32 = 1 << 2;
    /// Mapeamento compartilhado
    pub const SHARED: u32 = 1 << 3;
    /// Mapeamento privado (COW)
    pub const PRIVATE: u32 = 1 << 4;
    /// Endereço fixo (obrigatório)
    pub const FIXED: u32 = 1 << 5;
}

/// Flags para criação de porta IPC
pub mod port_flags {
    /// Porta para envio apenas
    pub const SEND_ONLY: u32 = 1 << 0;
    /// Porta para recebimento apenas
    pub const RECV_ONLY: u32 = 1 << 1;
    /// Porta bidirecional
    pub const BIDIRECTIONAL: u32 = 0;
}

/// Flags para mensagens IPC
pub mod msg_flags {
    /// Não bloquear se porta cheia
    pub const NONBLOCK: u32 = 1 << 0;
    /// Mensagem urgente (prioridade)
    pub const URGENT: u32 = 1 << 1;
}

/// Tipos de clock para SYS_CLOCK_GET
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockId {
    /// Tempo real (wall clock)
    Realtime = 0,
    /// Tempo monotônico (desde boot)
    Monotonic = 1,
    /// Tempo de CPU do processo
    ProcessCpu = 2,
    /// Tempo de CPU da thread
    ThreadCpu = 3,
}

/// Estrutura de tempo (nanosegundos)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeSpec {
    /// Segundos
    pub seconds: u64,
    /// Nanosegundos (0-999_999_999)
    pub nanoseconds: u32,
    /// Padding para alinhamento
    pub _pad: u32,
}

impl TimeSpec {
    /// Cria TimeSpec zerado
    pub const fn zero() -> Self {
        Self {
            seconds: 0,
            nanoseconds: 0,
            _pad: 0,
        }
    }

    /// Cria TimeSpec a partir de milissegundos
    pub fn from_millis(ms: u64) -> Self {
        Self {
            seconds: ms / 1000,
            nanoseconds: ((ms % 1000) * 1_000_000) as u32,
            _pad: 0,
        }
    }

    /// Converte para milissegundos
    pub fn to_millis(&self) -> u64 {
        self.seconds * 1000 + (self.nanoseconds / 1_000_000) as u64
    }
}
