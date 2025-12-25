//! Memory Management Subsystem
//! ===========================
//!
//! Este módulo concentra **todo o gerenciamento de memória do kernel**.
//! Ele define a arquitetura, responsabilidades e ordem de inicialização
//! dos componentes de memória do Redstone OS.
//!
//! O design segue princípios clássicos de kernels modernos:
//! - Separação clara entre memória física e virtual
//! - Controle explícito de paginação
//! - Inicialização previsível em early-kernel
//! - Zero dependência de runtime externo
//!
//! ---------------------------------------------------------------------
//! VISÃO GERAL DOS SUBMÓDULOS
//! ---------------------------------------------------------------------
//!
//! - `pmm` — **Physical Memory Manager**
//!   Gerencia frames físicos de 4 KiB usando bitmap.
//!   Responsável por:
//!   - interpretar o memory map do bootloader
//!   - rastrear frames livres/ocupados
//!   - fornecer frames para page tables, heap e outros subsistemas
//!
//! - `vmm` — **Virtual Memory Manager**
//!   Gerencia page tables x86_64 (PML4 → PDPT → PD → PT).
//!   Responsável por:
//!   - criar mapeamentos Virtual → Físico
//!   - resolver conflitos com huge pages (2 MiB)
//!   - manter o scratch slot para zeragem segura de frames
//!
//! - `heap` — **Kernel Heap Allocator**
//!   Implementa um alocador global (`GlobalAlloc`) baseado em bump allocator.
//!   Permite uso de `Box`, `Vec`, `String` dentro do kernel.
//!
//! ---------------------------------------------------------------------
//! MODELO DE MEMÓRIA DO KERNEL
//! ---------------------------------------------------------------------
//!
//! Fluxo de dependência (importante):
//!
//! ```text
//! PMM  ──▶ fornece frames físicos
//!  │
//!  ▼
//! VMM  ──▶ cria mapeamentos e page tables
//!  │
//!  ▼
//! Heap ──▶ aloca memória virtual usando VMM + PMM
//! ```
//!
//! ❗ O heap depende de PMM e VMM.
//! ❗ O VMM depende do PMM.
//! ❗ A ordem de inicialização NÃO é opcional.
//!
//! ---------------------------------------------------------------------
//! PREVENÇÃO DE DEADLOCK
//! ---------------------------------------------------------------------
//!
//! Um problema clássico de kernel:
//!
//! - O Heap precisa:
//!   - alocar frames (PMM)
//!   - mapear páginas (VMM)
//!
//! - O VMM, por sua vez, também pode precisar:
//!   - alocar frames (PMM)
//!
//! Se cada subsistema adquirir seu próprio lock, ocorre DEADLOCK.
//!
//! SOLUÇÃO ADOTADA:
//! - O lock do PMM é adquirido **uma única vez** no nível superior
//! - Esse lock é passado explicitamente para quem precisar
//! - Nunca há lock aninhado entre PMM ↔ VMM ↔ Heap
//!
//! Resultado:
//! - Sem deadlock
//! - Fluxo explícito
//! - Comportamento determinístico
//!
//! ---------------------------------------------------------------------
//! RESPONSABILIDADE DESTE MÓDULO
//! ---------------------------------------------------------------------
//!
//! Este módulo:
//! - Define a ordem correta de inicialização
//! - Garante isolamento entre subsistemas
//! - Atua como ponto único de entrada do sistema de memória
//!
//! Ele NÃO:
//! - implementa lógica específica de paginação
//! - implementa política de alocação avançada
//! - toma decisões de layout além do necessário
//!
//! ---------------------------------------------------------------------
//! GARANTIAS
//! ---------------------------------------------------------------------
//!
//! Após `memory::init()`:
//! - PMM está operacional
//! - VMM está operacional
//! - Heap do kernel está pronto
//! - `Box`, `Vec`, `String` podem ser usados com segurança
//!
//! Se algo falhar aqui, o kernel NÃO deve continuar.
//!
//! ---------------------------------------------------------------------

pub mod addr;
pub mod error;
pub mod heap;
pub mod pmm;
pub mod test;
pub mod vmm;

// Re-exports para conveniência
pub use addr::{is_phys_accessible, phys_to_virt, try_phys_to_virt, virt_to_phys};
pub use error::{MmError, MmResult};
pub use test::run_memory_tests;

/// Inicializa completamente o subsistema de memória do kernel.
///
/// Esta função é o **ponto único de inicialização** da memória.
/// Deve ser chamada exatamente uma vez, durante o boot inicial.
///
/// ------------------------------------------------------------------
/// ORDEM DE INICIALIZAÇÃO
/// ------------------------------------------------------------------
///
/// 1. **PMM (Physical Memory Manager)**
///    - Processa o memory map fornecido pelo bootloader
///    - Inicializa o bitmap de frames físicos
///
/// 2. **VMM (Virtual Memory Manager)**
///    - Lê o CR3 atual
///    - Inicializa o scratch slot
///    - Prepara infraestrutura de page tables
///
/// 3. **Heap do Kernel**
///    - Reserva região virtual do heap
///    - Aloca frames físicos
///    - Cria mapeamentos página por página
///
/// ------------------------------------------------------------------
/// DEADLOCK: COMO EVITAMOS
/// ------------------------------------------------------------------
///
/// O heap precisa usar PMM + VMM simultaneamente.
/// Para evitar deadlock:
///
/// - O lock do PMM é adquirido AQUI
/// - O heap recebe o PMM já bloqueado
/// - O heap repassa o PMM ao VMM quando necessário
///
/// Nenhum subsistema tenta adquirir o lock do PMM por conta própria
/// durante esta fase.
///
/// ------------------------------------------------------------------
/// SAFETY
/// ------------------------------------------------------------------
///
/// - Deve ser chamada apenas uma vez
/// - `boot_info` deve ser válido e imutável
/// - Executada em early-kernel (single core)
///
pub fn init(boot_info: &'static crate::core::handoff::BootInfo) {
    crate::kinfo!("(MM) Inicializando subsistema de memória...");

    // 1. Physical Memory Manager
    // Interpreta o memory map e configura o bitmap de frames
    unsafe {
        pmm::FRAME_ALLOCATOR.lock().init(boot_info);
    }

    crate::kinfo!("(MM) PMM OK, iniciando VMM...");

    // 2. Virtual Memory Manager
    // Inicializa paginação e scratch slot
    unsafe {
        vmm::init(boot_info);
    }

    crate::kinfo!("(MM) VMM OK, iniciando Heap...");

    // 3. Kernel Heap
    // Lock do PMM é adquirido aqui para evitar deadlock
    let mut pmm_lock = pmm::FRAME_ALLOCATOR.lock();
    if !heap::init_heap(&mut *pmm_lock) {
        panic!("(MM) Falha crítica ao inicializar heap!");
    }
    drop(pmm_lock); // Liberar lock antes dos testes

    crate::kinfo!("(MM) Subsistema de memória inicializado com sucesso!");

    // 4. Testes de memória (apenas quando verbose_logs está habilitado)
    #[cfg(feature = "verbose_logs")]
    {
        crate::kinfo!("(MM) Executando testes de memória (verbose_logs)...");
        test::run_memory_tests();
    }
}
