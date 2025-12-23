//! Estrutura de Contexto da CPU.
//!
//! Armazena o estado dos registradores que precisam ser preservados
//! durante uma troca de contexto (Callee-saved registers).

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
