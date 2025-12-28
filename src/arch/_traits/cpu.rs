//! Interface Abstrata de CPU (HAL).
//!
//! Define as operações fundamentais que qualquer arquitetura (x86_64, AArch64, RISC-V)
//! deve implementar para suportar o Kernel Redstone.
//!
//! # Contrato de Segurança
//! Implementações devem garantir isolamento e atomicidade onde documentado.
//!
//! # Padrão
//! - Suporte a SMP (Symmetric Multiprocessing) desde o design.
//! - Separação clara entre operações privilegiadas e utilitários.
//! - Tipagem forte para IDs e endereços.

use crate::sys::Errno;

/// ID físico ou lógico de um núcleo de processamento.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreId(pub u32);

/// Interface de operações de CPU.
/// Métodos estáticos operam no núcleo *atual* (caller).
pub trait CpuOps {
    // --- Identidade & Topologia ---

    /// Retorna o ID do núcleo atual.
    ///
    /// # Performance
    /// Deve ser uma operação O(1), geralmente lendo registrador local (GS base em x64, MPIDR em ARM).
    fn current_id() -> CoreId;

    /// Retorna true se este é o processador de boot (Bootstrap Processor).
    fn is_bsp() -> bool;

    // --- Controle de Execução ---

    /// Para a execução até a próxima interrupção (instrução HLT/WFI).
    /// Economiza energia em loops ociosos.
    ///
    /// # Safety
    /// Seguro de chamar, mas o caller deve estar ciente de que a execução para.
    fn halt();

    /// Dica para a CPU que estamos em um spinloop (instrução PAUSE/YIELD).
    /// Melhora performance e reduz consumo em hyperthreading.
    fn relax();

    /// Barreira de memória completa (Read/Write Fence).
    /// Garante que operações de memória anteriores completem antes das posteriores.
    /// Essencial para drivers e sincronização multicore.
    fn memory_fence();

    // --- Gerenciamento de Interrupções ---

    /// Desabilita interrupções locais (CLI).
    ///
    /// # Safety
    /// Crítico. Deve ser usado apenas quando necessário atomicidade absoluta no núcleo.
    /// Geralmente encapsulado por primitivas de sincronização (Mutex).
    unsafe fn disable_interrupts();

    /// Habilita interrupções locais (STI).
    ///
    /// # Safety
    /// Perigoso se chamado dentro de uma seção crítica que assume atomicidade.
    unsafe fn enable_interrupts();

    /// Verifica se as interrupções estão habilitadas (RFLAGS.IF).
    fn are_interrupts_enabled() -> bool;

    /// Entra em loop infinito de halt com interrupções desabilitadas.
    /// Usado em pânicos irrecuperáveis para congelar o núcleo.
    fn hang() -> ! {
        // SAFETY: Estamos em estado terminal. Desabilitar tudo é a única opção segura.
        unsafe { Self::disable_interrupts(); }
        loop {
            Self::halt();
        }
    }

    // --- IPI (Inter-Processor Interrupts) - Preparação para SMP ---

    /// Envia uma interrupção para outro núcleo.
    ///
    /// # Arguments
    /// * `target`: ID do núcleo alvo.
    /// * `vector`: Número do vetor de interrupção.
    fn send_ipi(target: CoreId, vector: u8) -> Result<(), Errno>;

    /// Envia uma interrupção para todos os núcleos, exceto o atual.
    /// Útil para TLB Shootdown ou Panic global.
    fn broadcast_ipi(vector: u8) -> Result<(), Errno>;
}