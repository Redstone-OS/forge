//! # CPU Execution Context
//!
//! A estrutura `Context` representa o estado mínimo da CPU que deve ser preservado
//! ao interromper uma tarefa (preempção) ou realizar uma chamada de sistema.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Callee-Saved Registers:** Armazena apenas os registradores que a ABI exige que sejam preservados por quem chama uma função.
//! - **Instruction Pointer:** Armazena o ponto de retorno (`RIP`).
//! - **Flags:** Armazena o `RFLAGS` para preservar interrupções e status aritmético.
//!
//! ## 🏗️ Arquitetura: System V AMD64 ABI
//! O Redstone OS segue estritamente a convenção de chamada System V para x86_64:
//! - **Preservados (Callee-saved):** `RBX`, `RBP`, `R12`, `R13`, `R14`, `R15`, `RSP` (via stack switch).
//! - **Voláteis (Caller-saved):** `RAX`, `RCX`, `RDX`, `RSI`, `RDI`, `R8`..`R11` (assumimos que o compilador ou a interrupção já salvou se necessário).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Minimalismo:** A estrutura tem apenas 160 bytes (aproximadamente, se alinhada), permitindo contexto switch rápido.
//! - **ABI Compliance:** Evita salvar registradores desnecessários (como `RAX`), confiando que o código Rust já lida com eles.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **FPU/SSE Ignorado:** A estrutura **NÃO** contém espaço para registadores XMM/YMM/ZMM.
//!   - *Consequência:* Se duas threads usarem float/vector instructions, uma sobrescreverá os dados da outra.
//! - **Interrupções:** O valor padrão de `rflags` é hardcoded (0x202). Deveria usar constantes definidas.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical)** Expandir `Context` para suportar **XSAVE / FXSAVE** (512+ bytes).
//!   - *Motivo:* Suporte a aplicações modernas, criptografia e media processing.
//! - [ ] **TODO: (Debug)** Adicionar suporte para *Stack Trace* automático a partir do `RBP` salvo no contexto.

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    // Registradores preservados pela ABI (System V AMD64)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,

    // Instruction Pointer (RIP)
    pub rip: u64,

    // RFLAGS (Status da CPU)
    pub rflags: u64,
}

impl Context {
    /// Cria um contexto vazio.
    pub const fn empty() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
            rflags: 0x202, // Interrupts enabled (IF=1), Reserved bit 1=1
        }
    }

    /// Cria um contexto inicial para uma nova tarefa.
    ///
    /// # Arguments
    /// * `entry_point`: Endereço da função a ser executada.
    /// * `stack_top`: Topo da stack (endereço mais alto, pois stack cresce para baixo).
    pub fn new(entry_point: u64, _stack_top: u64) -> Self {
        Self {
            rip: entry_point,
            rflags: 0x202, // IF=1
            ..Default::default()
        }
    }
}
