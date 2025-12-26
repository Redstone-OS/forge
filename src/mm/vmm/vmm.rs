//! # Virtual Memory Manager (VMM)
//!
//! O `VMM` implementa a paginação de 4 Níveis (x86_64 PML4) e gerencia o espaço de endereçamento virtual do kernel.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Page Table Management:** Cria, modifica e navega na hierarquia PML4 → PDPT → PD → PT.
//! - **Memory Mapping:** Mapeia endereços físicos arbitrários em virtuais (`map_page`).
//! - **Fine-Grained Access:** Divide "Huge Pages" (2MiB) em 512 páginas de 4KiB sob demanda para permitir proteção granular.
//!
//! ## 🏗️ Arquitetura Singular: Scratch Slot & Huge Splitting
//! Diferente de VMMs acadêmicos, este VMM resolve problemas reais de hardware moderno:
//!
//! 1. **Scratch Slot:** Uma região virtual fixa (`0xFFFF_FE00_...`) usada para mapear temporariamente frames físicos.
//!    - *Por que?* Para zerar uma nova Page Table antes de inseri-la na hierarquia, sem depender de "Identity Map" (que pode não cobrir toda a RAM).
//! 2. **Auto-Splitting:** Se `map_page` encontra uma Huge Page (2MB) no caminho, ele a converte atomicamente em uma tabela de páginas menores.
//!    - *Por que?* Bootloaders mapeiam 0-4GB como Huge Pages para performance. O kernel precisa de granularidade 4KB para `MPROTECT` e `Guard Pages`.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Isolamento de Boot:** O uso do Scratch Slot desacopla a inicialização do VMM das decisões do bootloader.
//! - **Robustez:** A lógica de *Splitting* permite que o kernel refine permissões de memória (ex: tornar `.rodata` Read-Only) mesmo se o bootloader entregou tudo como RWX Huge Pages.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **TLB Shootdown Inexistente:** Em multicore, alterar uma page table aqui **não** invalida o TLB de outros CPUs.
//!   - *Consequência:* Risco gravíssimo de corrupção de memória em SMP.
//! - **Ausência de `unmap`:** Atualmente o VMM só sabe mapear. Não há lógica para remover mapeamentos e liberar frames das page tables intermediárias.
//! - **Hardcoded Offsets:** Os índices PML4 (Kernel, Heap, Scratch) são constantes mágicas que devem bater com o `Ignite`. Desalinhamento = Crash.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/SMP)** Implementar **TLB Shootdown**.
//!   - *Ação:* Enviar IPI (Inter-Processor Interrupt) para todos os cores executarem `invlpg` ao alterar mapeamentos globais.
//! - [ ] **TODO: (Management)** Implementar `unmap_page(virt_addr)`.
//!   - *Requisito:* Liberar frames físicos se a Page Table ficar vazia (Reclaim).
//! - [ ] **TODO: (Security)** Implementar bits **NX (No-Execute)** e **USER/SUPERVISOR** rigorosos.
//!   - *Alvo:* Garantir que Heap/Stack não sejam executáveis (W^X).
//! - [ ] **TODO: (Feature)** Suporte a **5-Level Paging** (Ice Lake+).
//!   - *Impacto:* Permitir endereçamento virtual acima de 256 TiB (futuro distante).
//!
//! ----------------------------------------------------------------------
//! ARQUITETURA DE PAGINAÇÃO x86_64
//! ----------------------------------------------------------------------
//!
//! Endereço virtual canônico (48 bits significativos):
//!
//! ```text
//! ┌────────────┬────────────┬────────────┬────────────┬────────────┐
//! │ PML4 Index │ PDPT Index │  PD Index  │  PT Index  │   Offset   │
//! │  (9 bits)  │  (9 bits)  │  (9 bits)  │  (9 bits)  │  (12 bits) │
//! └────────────┴────────────┴────────────┴────────────┴────────────┘
//!      bits 47-39    38-30       29-21        20-12         11-0
//! ```
//!
//! Cada nível contém 512 entradas (512 × 8 bytes = 4 KiB).
//! A tradução é resolvida do nível mais alto (PML4) até o PT,
//! exceto quando huge pages estão presentes.
//!
//! ----------------------------------------------------------------------
//! PROBLEMA CLÁSSICO: HUGE PAGES vs PAGE TABLES (4 KiB)
//! ----------------------------------------------------------------------
//!
//! O bootloader cria um **identity map de 0–4 GiB usando huge pages (2 MiB)**.
//! Isso reduz o número de page tables e acelera o boot.
//!
//! ⚠️ PROBLEMA CRÍTICO:
//! Quando uma entrada de PD possui a flag `PAGE_HUGE`:
//!
//! - o CPU **ignora completamente** o nível PT;
//! - qualquer tentativa de criar page tables de 4 KiB dentro dessa região
//!   é ignorada;
//! - acessos resultam em **General Protection Fault (GPF)**.
//!
//! Esse comportamento causava crashes ao:
//! - alocar page tables dinamicamente;
//! - zerar frames físicos recém-alocados;
//! - mapear regiões internas ao identity map.
//!
//! ----------------------------------------------------------------------
//! SOLUÇÃO ARQUITETURAL ADOTADA
//! ----------------------------------------------------------------------
//!
//! A solução combina **isolamento** + **adaptação dinâmica**:
//!
//! 1. SCRATCH SLOT
//!    - Região virtual fixa, isolada e garantidamente NÃO huge;
//!    - usada exclusivamente para mapear temporariamente frames físicos;
//!    - permite zerar frames antes de inseri-los na hierarquia de page tables;
//!    - elimina o problema "chicken-and-egg" da inicialização.
//!
//! 2. SPLIT AUTOMÁTICO DE HUGE PAGES
//!    - ao detectar uma entrada com `PAGE_HUGE`:
//!      - uma nova Page Table (PT) é alocada;
//!      - o mapeamento da huge page (2 MiB) é replicado em
//!        512 páginas de 4 KiB;
//!      - a entrada do PD é substituída por um ponteiro para a nova PT;
//!      - o comportamento funcional original é preservado.
//!
//! Resultado:
//! - o kernel pode criar page tables 4 KiB em qualquer região;
//! - nenhuma dependência de identity map para operações internas;
//! - eliminação definitiva de GPFs causados por huge pages.
//!
//! ----------------------------------------------------------------------
//! REGIÕES IMPORTANTES DO ESPAÇO VIRTUAL
//! ----------------------------------------------------------------------
//!
//! Convenção atual de layout:
//!
//! - PML4[0..3]   → Identity map (0–4 GiB, huge pages)
//! - PML4[288]    → Heap do kernel
//! - PML4[511]    → Higher-half do kernel
//! - PML4[508]    → SCRATCH SLOT (isolado, limpo, seguro)
//!
//! ⚠️ Qualquer alteração nesses índices **DEVE** ser refletida
//! no bootloader (Ignite). Desalinhamento aqui causa falhas
//! difíceis de diagnosticar.
//!
//! ----------------------------------------------------------------------
//! CONTRATOS E INVARIANTES (NÃO QUEBRE)
//! ----------------------------------------------------------------------
//!
//! 1. O bootloader deve:
//!    - fornecer identity map 0–4 GiB usando huge pages;
//!    - reservar corretamente o scratch slot no PML4.
//!
//! 2. `init()` deve ser chamada:
//!    - exatamente uma vez;
//!    - em early-boot;
//!    - antes de qualquer uso do heap.
//!
//! 3. O PMM (Physical Memory Manager) deve estar operacional
//!    antes de qualquer tentativa de criar page tables.
//!
//! 4. Toda nova page table alocada é:
//!    - zerada antes do uso;
//!    - publicada na hierarquia apenas após estar consistente.
//!
//! ----------------------------------------------------------------------
//! SEGURANÇA E USO DE `unsafe`
//! ----------------------------------------------------------------------
//!
//! Este módulo utiliza `unsafe` por necessidade arquitetural:
//! - escrita direta em page tables;
//! - conversão de endereços físicos em ponteiros;
//! - manipulação explícita de CR3 e TLB.
//!
//! Medidas adotadas:
//! - `invlpg` após modificações relevantes;
//! - zeragem explícita de frames;
//! - isolamento do scratch slot.
//!
//! ----------------------------------------------------------------------
//! RISCOS CONHECIDOS
//! ----------------------------------------------------------------------
//!
//! - Scratch slot ausente ou mal configurado pode ativar fallback
//!   inseguro (escrita direta via identity map);
//! - Split de huge page pode falhar em condições severas de OOM;
//! - Em SMP, este código assume execução single-core
//!   (não há TLB shootdown).
//!
//! ----------------------------------------------------------------------
//! EXTENSÕES FUTURAS RECOMENDADAS
//! ----------------------------------------------------------------------
//!
//! - Migrar retornos `bool` / `0` para `Result<T, Error>`;
//! - Implementar `unmap_page()` e validações de mapeamento;
//! - Rollback seguro em split parcial de huge page;
//! - Protocolo de TLB shootdown para SMP.
//!
//! ----------------------------------------------------------------------
//! Abaixo deste ponto: implementação.
//! Comentários locais explicam decisões específicas.
use crate::mm::addr::{phys_to_virt, PhysAddr};
use crate::mm::error::{MmError, MmResult};
use crate::mm::pmm::{BitmapFrameAllocator, FRAME_ALLOCATOR};
use core::arch::asm;

// =============================================================================
// FLAGS DE PAGINAÇÃO x86_64
// =============================================================================
// Flags importadas de mm::config para garantir consistência em todo o sistema.
// Flags importadas de mm::config para garantir consistência em todo o sistema.
// Re-exportamos para manter compatibilidade com módulos que usam vmm::PAGE_...
pub use crate::mm::config::{
    PAGE_HUGE, PAGE_MASK, PAGE_NO_EXEC, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
};

// =============================================================================
// ESTADO GLOBAL DO VMM
// =============================================================================

/// Endereço físico da PML4 ativa (valor extraído do CR3 no `init()`).
static mut ACTIVE_PML4_PHYS: u64 = 0;

// =============================================================================
// SCRATCH SLOT — mecanismo para zeragem segura de frames
// =============================================================================
//
// O scratch slot é uma peça crítica do early-boot: fornece um endereço virtual
// limpo (garantido não coberto por huge pages) onde podemos mapear um frame físico
// temporariamente para operações primitivas (zerar, copiar).
//
// - O bootloader deve reservar esse índice PML4 e criar a estrutura PML4→PDPT→PD→PT
//   para o scratch slot antes de transferir controle ao kernel.
// - Aqui validamos e usamos essa PT; se não existir, o módulo registra aviso e
//   tenta fallback (menos seguro).
//
// Observação: se você mudar o índice do scratch, atualize o bootloader (ignite).
//
// Usamos u64 aqui para facilitar operações bitwise, mas a constante vem de config (usize).
const SCRATCH_VIRT: u64 = crate::mm::config::SCRATCH_VIRT as u64;
static mut SCRATCH_PT_PHYS: u64 = 0; // endereço físico da PT do scratch
static mut SCRATCH_READY: bool = false; // indica disponibilidade operacional

// =============================================================================
// INICIALIZAÇÃO DO VMM
// =============================================================================

/// Inicializa o VMM a partir do contexto fornecido pelo bootloader.
///
/// Pré-condições:
/// - O bootloader já estabeleceu identity map (0..4GiB) com huge pages.
/// - O bootloader criou a hierarquia do scratch slot no PML4 acordado.
///
/// Safety:
/// - Deve ser invocado **uma única vez** em early-boot.
/// - O caller deve garantir que o ambiente (CR3, boot_info) esteja consistente.
pub unsafe fn init(boot_info: &crate::core::handoff::BootInfo) {
    crate::kdebug!("(VMM) init: Iniciando...");

    // Lê CR3 (PML4 físico atual) e guarda para uso interno.
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3);
    ACTIVE_PML4_PHYS = cr3 & 0x000F_FFFF_FFFF_F000;

    crate::kdebug!(
        "(VMM) init: CR3={:#x}, PML4_PHYS={:#x}",
        cr3,
        ACTIVE_PML4_PHYS
    );
    crate::ktrace!("(VMM) init: SCRATCH_VIRT={:#x}", SCRATCH_VIRT);

    // Valida e inicializa o scratch slot para operações de zeragem.
    init_scratch_slot();

    if SCRATCH_READY {
        crate::kdebug!(
            "(VMM) init: Scratch slot pronto em PT {:#x}",
            SCRATCH_PT_PHYS
        );
    } else {
        crate::kwarn!("(VMM) init: Scratch slot NÃO disponível - usando fallback");
    }

    // (nota) O `boot_info` pode ser usado para logging adicional
    let _ = boot_info;
    crate::kdebug!("(VMM) init: OK");
}

/// Localiza e valida a Page Table do scratch slot criada pelo bootloader.
///
/// Esta função percorre PML4→PDPT→PD→PT e verifica que:
/// - entradas estão marcadas PRESENT;
/// - a entrada do PD não é uma HUGE PAGE (o que invalidaria a PT).
///
/// Se qualquer verificação falhar, a função registra o problema e marca
/// `SCRATCH_READY = false` (fallback ou correção manual necessária).
unsafe fn init_scratch_slot() {
    crate::kdebug!("(VMM) Inicializando scratch slot...");

    let pml4_idx = ((SCRATCH_VIRT >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((SCRATCH_VIRT >> 30) & 0x1FF) as usize;
    let pd_idx = ((SCRATCH_VIRT >> 21) & 0x1FF) as usize;

    // Usar phys_to_virt para acessar PML4
    let pml4: *const u64 = phys_to_virt(PhysAddr::new(ACTIVE_PML4_PHYS)).as_ptr();
    let pml4_entry = *pml4.add(pml4_idx);

    if pml4_entry & PAGE_PRESENT == 0 {
        crate::kwarn!("(VMM) Scratch: PML4[{}] não presente", pml4_idx);
        SCRATCH_READY = false;
        return;
    }

    let pdpt_phys = pml4_entry & PAGE_MASK;
    let pdpt: *const u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_ptr();
    let pdpt_entry = *pdpt.add(pdpt_idx);

    if pdpt_entry & PAGE_PRESENT == 0 {
        crate::kwarn!("(VMM) Scratch: PDPT[{}] não presente", pdpt_idx);
        SCRATCH_READY = false;
        return;
    }

    let pd_phys = pdpt_entry & PAGE_MASK;
    let pd: *const u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_ptr();
    let pd_entry = *pd.add(pd_idx);

    if pd_entry & PAGE_PRESENT == 0 {
        crate::kwarn!("(VMM) Scratch: PD[{}] não presente", pd_idx);
        SCRATCH_READY = false;
        return;
    }

    if pd_entry & PAGE_HUGE != 0 {
        crate::kwarn!("(VMM) Scratch: PD[{}] é huge page!", pd_idx);
        SCRATCH_READY = false;
        return;
    }

    SCRATCH_PT_PHYS = pd_entry & PAGE_MASK;
    SCRATCH_READY = true;
    crate::kdebug!("(VMM) Scratch slot OK: PT em {:#x}", SCRATCH_PT_PHYS);
}

// =============================================================================
// ZERAGEM DE FRAMES FÍSICOS (USANDO SCRATCH OU IDENTITY MAP)
// =============================================================================

/// Zera um frame físico com segurança.
///
/// # Estratégia
///
/// 1. Se SCRATCH_READY: usa scratch slot (mais seguro, funciona para qualquer endereço)
/// 2. Se não: usa phys_to_virt via identity map (só funciona para phys < 4GB)
///
/// # Returns
///
/// - `Ok(())` se o frame foi zerado com sucesso
/// - `Err(MmError::ScratchNotReady)` se scratch indisponível e phys >= 4GB
unsafe fn zero_frame_via_scratch(phys: u64) -> MmResult<()> {
    // Usar scratch slot se disponível (método preferido)
    if SCRATCH_READY {
        let pt_idx = ((SCRATCH_VIRT >> 12) & 0x1FF) as usize;
        let pt_ptr: *mut u64 = phys_to_virt(PhysAddr::new(SCRATCH_PT_PHYS)).as_mut_ptr();
        let pte_ptr = pt_ptr.add(pt_idx);

        // Salvar PTE original usando assembly (evita SSE)
        let original_pte: u64;
        core::arch::asm!(
            "mov {0}, [{1}]",
            out(reg) original_pte,
            in(reg) pte_ptr,
            options(nostack, preserves_flags, readonly)
        );

        // Mapear frame no scratch slot usando assembly
        let temp_pte = (phys & PAGE_MASK) | PAGE_PRESENT | PAGE_WRITABLE;
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) pte_ptr,
            in(reg) temp_pte,
            options(nostack, preserves_flags)
        );
        asm!("invlpg [{}]", in(reg) SCRATCH_VIRT, options(nostack, preserves_flags));

        // Zerar via endereço virtual do scratch usando while + assembly
        let base = SCRATCH_VIRT as *mut u64;
        let mut i = 0usize;
        let zero_val = 0u64;
        while i < 512 {
            let ptr = base.add(i);
            core::arch::asm!(
                "mov [{0}], {1}",
                in(reg) ptr,
                in(reg) zero_val,
                options(nostack, preserves_flags)
            );
            i += 1;
        }

        // Restaurar PTE original usando assembly
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) pte_ptr,
            in(reg) original_pte,
            options(nostack, preserves_flags)
        );
        asm!("invlpg [{}]", in(reg) SCRATCH_VIRT, options(nostack, preserves_flags));

        return Ok(());
    }

    // Fallback: usar identity map via phys_to_virt (só funciona para < 4GB)
    if !crate::mm::addr::is_phys_accessible(PhysAddr::new(phys)) {
        crate::kerror!(
            "(VMM) zero_frame: phys {:#x} inacessível sem scratch!",
            phys
        );
        return Err(MmError::ScratchNotReady);
    }

    crate::ktrace!("(VMM) zero_frame: usando identity map para {:#x}", phys);
    let ptr: *mut u64 = phys_to_virt(PhysAddr::new(phys)).as_mut_ptr();
    let mut i = 0usize;
    let zero_val = 0u64;
    while i < 512 {
        let entry_ptr = ptr.add(i);
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) entry_ptr,
            in(reg) zero_val,
            options(nostack, preserves_flags)
        );
        i += 1;
    }

    Ok(())
}

// =============================================================================
// MAPEAMENTO DE PÁGINAS VIRTUAIS (APIs PÚBLICAS)
// =============================================================================

/// Mapeia `virt_addr` → `phys_addr` com `flags`.
///
/// - Esta versão adquire o lock global do FRAME_ALLOCATOR internamente.
/// - Para evitar deadlocks em caminhos que já têm o lock do PMM, use
///   `map_page_with_pmm()` passando o PMM explicitamente.
///
/// Retorna `Ok(())` em sucesso, `Err` em OOM ou erro.
pub unsafe fn map_page(virt_addr: u64, phys_addr: u64, flags: u64) -> MmResult<()> {
    let mut pmm = FRAME_ALLOCATOR.lock();
    map_page_with_pmm(virt_addr, phys_addr, flags, &mut *pmm)
}

/// Mapeia `virt_addr` → `phys_addr` usando o PMM fornecido.
///
/// Algoritmo:
/// 1. Calcula índices PML4/PDPT/PD/PT a partir de `virt_addr`.
/// 2. Para cada nível, garante que exista uma tabela (alocando via PMM se necessário).
/// 3. Se encontrar uma entrada PD marcada como HUGE, faz SPLIT para PT (512 entradas).
/// 4. Escreve a PTE final e invalida TLB.
///
/// Use esta função quando você já segurou o lock do PMM (evita deadlock).
pub unsafe fn map_page_with_pmm(
    virt_addr: u64,
    phys_addr: u64,
    flags: u64,
    pmm: &mut BitmapFrameAllocator,
) -> MmResult<()> {
    // DEBUG: Log de entrada apenas na primeira página
    static mut FIRST_MAP: bool = true;
    let is_first = FIRST_MAP;
    if is_first {
        FIRST_MAP = false;
        crate::ktrace!(
            "(VMM) map_page_with_pmm: virt={:#x} phys={:#x}",
            virt_addr,
            phys_addr
        );
    }

    // Índices da hierarquia (9 bits cada)
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;

    if is_first {
        crate::ktrace!(
            "(VMM) indices: pml4={} pdpt={} pd={} pt={}",
            pml4_idx,
            pdpt_idx,
            pd_idx,
            pt_idx
        );
    }

    // Ponteiro para a PML4 atual via phys_to_virt
    let pml4_ptr: *mut u64 = phys_to_virt(PhysAddr::new(ACTIVE_PML4_PHYS)).as_mut_ptr();

    if is_first {
        crate::ktrace!("(VMM) pml4_ptr = {:p}", pml4_ptr);
    }

    let pml4_entry = &mut *pml4_ptr.add(pml4_idx);

    if is_first {
        crate::ktrace!(
            "(VMM) pml4_entry = {:#x}, chamando ensure_table...",
            *pml4_entry
        );
    }

    // Garantir PDPT existe
    let pdpt_phys = ensure_table_entry_with_pmm(pml4_entry, pmm);
    if pdpt_phys == 0 {
        crate::kerror!("(VMM) map_page: falha ao criar PDPT para {:#x}", virt_addr);
        return Err(MmError::OutOfMemory);
    }

    let pdpt_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_mut_ptr();
    let pdpt_entry = &mut *pdpt_ptr.add(pdpt_idx);

    // Garantir PD existe
    let pd_phys = ensure_table_entry_with_pmm(pdpt_entry, pmm);
    if pd_phys == 0 {
        crate::kerror!("(VMM) map_page: falha ao criar PD para {:#x}", virt_addr);
        return Err(MmError::OutOfMemory);
    }

    let pd_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_mut_ptr();
    let pd_entry = &mut *pd_ptr.add(pd_idx);

    // Garantir PT existe (pode exigir split de huge page)
    let pt_phys = ensure_table_entry_with_pmm(pd_entry, pmm);
    if pt_phys == 0 {
        crate::kerror!("(VMM) map_page: falha ao criar PT para {:#x}", virt_addr);
        return Err(MmError::OutOfMemory);
    }

    let pt_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pt_phys)).as_mut_ptr();
    let pt_entry = &mut *pt_ptr.add(pt_idx);

    // Escrever PTE final (endereço físico + flags + PRESENT)
    *pt_entry = (phys_addr & PAGE_MASK) | flags | PAGE_PRESENT;

    // Barreira de memória: garante que a escrita da PTE seja visível antes de invlpg
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    // Invalida TLB para o endereço mapeado
    asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));

    Ok(())
}

// =============================================================================
// GERENCIAMENTO DE ENTRADAS DE PAGE TABLES (CORE)
// =============================================================================

/// Garante que a entrada de uma page-table (`entry`) aponte para uma tabela válida.
///
/// Comportamento:
/// - Se `entry` já estiver presente e não for HUGE, retorna o endereço da tabela.
/// - Se `entry` for HUGE, realiza SPLIT: aloca uma PT e replica o mapeamento da huge page
///   em 512 entradas de 4 KiB, atualizando a entrada para apontar para a nova PT.
/// - Se `entry` não estiver presente, aloca uma nova tabela (frame), zera-a e retorna seu endereço.
///
/// Retorna o endereço físico da tabela (PDPT/PD/PT conforme o nível), ou 0 em caso de OOM.
///
/// Safety:
/// - `entry` deve ser um ponteiro válido para uma entrada da page table.
/// - `pmm` deve estar inicializado e operante.
unsafe fn ensure_table_entry_with_pmm(entry: &mut u64, pmm: &mut BitmapFrameAllocator) -> u64 {
    // DEBUG: log apenas para primeiras chamadas
    static mut CALL_COUNT: usize = 0;
    let count = CALL_COUNT;
    CALL_COUNT += 1;
    let should_log = count < 5;

    if should_log {
        crate::ktrace!("(VMM) ensure_table[{}]: entry={:#x}", count, *entry);
    }

    // Caso: entrada já existe
    if *entry & PAGE_PRESENT != 0 {
        // Se for uma huge page, precisamos converter para uma PT de 4 KiB (split)
        if *entry & PAGE_HUGE != 0 {
            // --- SPLIT DE HUGE PAGE ---
            crate::kdebug!("(VMM) Splitting huge page...");

            // Extraímos a base física da huge page (alinhada a 2 MiB).
            let huge_base = *entry & 0x000F_FFFF_FFE0_0000;

            // Preservamos flags relevantes, exceto o bit HUGE.
            let old_flags = *entry & 0xFFF;

            // Aloca frame para a nova PT (4 KiB)
            let frame = match pmm.allocate_frame() {
                Some(f) => f,
                None => {
                    crate::kerror!("(VMM) OOM ao fazer split de huge page!");
                    return 0;
                }
            };
            let pt_phys = frame.addr();

            // Zera a página que conterá a nova PT antes de escrever entradas.
            if let Err(e) = zero_frame_via_scratch(pt_phys) {
                crate::kerror!("(VMM) Falha ao zerar PT para split: {}", e);
                return 0;
            }

            // Preenche a PT replicando o mapeamento da huge page em 512 entradas.
            let pt: *mut u64 = phys_to_virt(PhysAddr::new(pt_phys)).as_mut_ptr();
            let new_flags = (old_flags & !PAGE_HUGE) | PAGE_PRESENT | PAGE_WRITABLE;

            let mut j = 0usize;
            while j < 512 {
                let page_phys = huge_base + (j as u64 * 4096);
                let entry_val = page_phys | new_flags;
                let entry_ptr = pt.add(j);
                core::arch::asm!(
                    "mov [{0}], {1}",
                    in(reg) entry_ptr,
                    in(reg) entry_val,
                    options(nostack, preserves_flags)
                );
                j += 1;
            }

            // Atualiza a entrada do PD para apontar para a nova PT (removendo HUGE).
            *entry = pt_phys | PAGE_PRESENT | PAGE_WRITABLE;

            // Invalidar TLB para toda a região anteriormente mapeada como huge page.
            let mut k = 0u64;
            while k < 512 {
                let vaddr = huge_base + (k * 4096);
                asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
                k += 1;
            }

            crate::kdebug!("(VMM) Huge page split OK: nova PT em {:#x}", pt_phys);
            return pt_phys;
        }

        // Entrada presente e não-huge: garantir flags de acesso e retornar endereço.
        *entry |= PAGE_USER | PAGE_WRITABLE;
        if should_log {
            crate::ktrace!(
                "(VMM) ensure_table[{}]: entry presente, phys={:#x}",
                count,
                *entry & PAGE_MASK
            );
        }
        return *entry & PAGE_MASK;
    }

    // Caso: entrada ausente — aloca nova tabela (frame de 4 KiB)
    if should_log {
        crate::ktrace!("(VMM) ensure_table[{}]: alocando nova tabela...", count);
    }

    let frame = match pmm.allocate_frame() {
        Some(f) => f,
        None => {
            crate::kerror!("(VMM) OOM ao alocar page table!");
            return 0;
        }
    };
    let phys = frame.addr();

    if should_log {
        crate::ktrace!(
            "(VMM) ensure_table[{}]: frame={:#x}, zerando...",
            count,
            phys
        );
    }

    // Zera a nova tabela (crítico para evitar leitura de lixo).
    if let Err(e) = zero_frame_via_scratch(phys) {
        crate::kerror!("(VMM) Falha ao zerar nova page table: {}", e);
        return 0;
    }

    // Configura a entrada para apontar para a nova tabela: PRESENT | WRITABLE | USER.
    *entry = phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    phys
}

/// Traduz um endereço virtual para físico.
/// Retorna None se não estiver mapeado.
pub fn translate_addr(virt_addr: u64) -> Option<u64> {
    unsafe {
        let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
        let pml4_ptr: *const u64 = phys_to_virt(PhysAddr::new(ACTIVE_PML4_PHYS)).as_ptr();
        let pml4_entry = *pml4_ptr.add(pml4_idx);
        if pml4_entry & PAGE_PRESENT == 0 {
            return None;
        }

        let pdpt_phys = pml4_entry & PAGE_MASK;
        let pdpt_ptr: *const u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_ptr();
        let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
        let pdpt_entry = *pdpt_ptr.add(pdpt_idx);
        if pdpt_entry & PAGE_PRESENT == 0 {
            return None;
        }

        // Huge page (1GB) check
        if pdpt_entry & PAGE_HUGE != 0 {
            let base = pdpt_entry & 0x000F_FFFF_C000_0000;
            let offset = virt_addr & 0x3FFF_FFFF;
            return Some(base + offset);
        }

        let pd_phys = pdpt_entry & PAGE_MASK;
        let pd_ptr: *const u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_ptr();
        let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
        let pd_entry = *pd_ptr.add(pd_idx);
        if pd_entry & PAGE_PRESENT == 0 {
            return None;
        }

        // Huge page (2MB) check
        if pd_entry & PAGE_HUGE != 0 {
            let base = pd_entry & 0x000F_FFFF_FFE0_0000;
            let offset = virt_addr & 0x1FFFFF;
            return Some(base + offset);
        }

        let pt_phys = pd_entry & PAGE_MASK;
        let pt_ptr: *const u64 = phys_to_virt(PhysAddr::new(pt_phys)).as_ptr();
        let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
        let pt_entry = *pt_ptr.add(pt_idx);
        if pt_entry & PAGE_PRESENT == 0 {
            return None;
        }

        let phys_base = pt_entry & PAGE_MASK;
        Some(phys_base + (virt_addr & 0xFFF))
    }
}

/// Traduz endereço virtual para físico e retorna as flags da página.
pub fn translate_addr_with_flags(virt_addr: u64) -> Option<(u64, u64)> {
    unsafe {
        let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
        let pml4_ptr: *const u64 = phys_to_virt(PhysAddr::new(ACTIVE_PML4_PHYS)).as_ptr();
        let pml4_entry = *pml4_ptr.add(pml4_idx);
        if pml4_entry & PAGE_PRESENT == 0 {
            return None;
        }

        let pdpt_phys = pml4_entry & PAGE_MASK;
        let pdpt_ptr: *const u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_ptr();
        let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
        let pdpt_entry = *pdpt_ptr.add(pdpt_idx);
        if pdpt_entry & PAGE_PRESENT == 0 {
            return None;
        }

        if pdpt_entry & PAGE_HUGE != 0 {
            let base = pdpt_entry & 0x000F_FFFF_C000_0000;
            let offset = virt_addr & 0x3FFF_FFFF;
            return Some((base + offset, pdpt_entry & 0xFFF));
        }

        let pd_phys = pdpt_entry & PAGE_MASK;
        let pd_ptr: *const u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_ptr();
        let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
        let pd_entry = *pd_ptr.add(pd_idx);
        if pd_entry & PAGE_PRESENT == 0 {
            return None;
        }

        if pd_entry & PAGE_HUGE != 0 {
            let base = pd_entry & 0x000F_FFFF_FFE0_0000;
            let offset = virt_addr & 0x1FFFFF;
            return Some((base + offset, pd_entry & 0xFFF));
        }

        let pt_phys = pd_entry & PAGE_MASK;
        let pt_ptr: *const u64 = phys_to_virt(PhysAddr::new(pt_phys)).as_ptr();
        let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
        let pt_entry = *pt_ptr.add(pt_idx);
        if pt_entry & PAGE_PRESENT == 0 {
            return None;
        }

        let phys_base = pt_entry & PAGE_MASK;
        Some((phys_base + (virt_addr & 0xFFF), pt_entry & 0xFFF))
    }
}

// =============================================================================
// UNMAP PAGE
// =============================================================================

/// Remove mapeamento de uma página virtual
///
/// # Descrição
///
/// Esta função remove o mapeamento de uma página virtual, retornando
/// o endereço físico que estava mapeado anteriormente.
///
/// # Safety
///
/// - A página deve estar mapeada
/// - O caller deve garantir que ninguém está usando a página
/// - O caller é responsável por liberar o frame físico se necessário
///
/// # Retorno
///
/// - `Ok(PhysAddr)` com o endereço físico que estava mapeado
/// - `Err(MmError::NotMapped)` se a página não estava mapeada
/// - `Err(MmError::HugePageNotSupported)` se for uma huge page
///
/// # Exemplo
///
/// ```rust
/// // Desmapear página
/// let phys = unsafe { unmap_page(0xDEAD_0000_0000)? };
///
/// // Liberar o frame físico
/// let frame = PhysFrame::from_start_address(phys);
/// pmm.deallocate_frame(frame);
/// ```
pub unsafe fn unmap_page(virt_addr: u64) -> MmResult<PhysAddr> {
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;

    // Acessar PML4
    let pml4_ptr: *mut u64 = phys_to_virt(PhysAddr::new(ACTIVE_PML4_PHYS)).as_mut_ptr();
    let pml4_entry = *pml4_ptr.add(pml4_idx);

    if pml4_entry & PAGE_PRESENT == 0 {
        return Err(MmError::NotMapped);
    }

    // Acessar PDPT
    let pdpt_phys = pml4_entry & PAGE_MASK;
    let pdpt_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_mut_ptr();
    let pdpt_entry = *pdpt_ptr.add(pdpt_idx);

    if pdpt_entry & PAGE_PRESENT == 0 {
        return Err(MmError::NotMapped);
    }

    // Verificar huge page 1GB
    if pdpt_entry & PAGE_HUGE != 0 {
        return Err(MmError::HugePageNotSupported);
    }

    // Acessar PD
    let pd_phys = pdpt_entry & PAGE_MASK;
    let pd_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_mut_ptr();
    let pd_entry = *pd_ptr.add(pd_idx);

    if pd_entry & PAGE_PRESENT == 0 {
        return Err(MmError::NotMapped);
    }

    // Verificar huge page 2MB
    if pd_entry & PAGE_HUGE != 0 {
        return Err(MmError::HugePageNotSupported);
    }

    // Acessar PT
    let pt_phys = pd_entry & PAGE_MASK;
    let pt_ptr: *mut u64 = phys_to_virt(PhysAddr::new(pt_phys)).as_mut_ptr();
    let pt_entry_ptr = pt_ptr.add(pt_idx);
    let pt_entry = *pt_entry_ptr;

    if pt_entry & PAGE_PRESENT == 0 {
        return Err(MmError::NotMapped);
    }

    // Extrair endereço físico antes de limpar
    let phys = pt_entry & PAGE_MASK;

    // Limpar entrada da page table
    *pt_entry_ptr = 0;

    // Invalidar TLB
    core::arch::asm!(
        "invlpg [{}]",
        in(reg) virt_addr,
        options(nostack, preserves_flags)
    );

    crate::ktrace!(
        "(VMM) unmap_page: virt={:#x} -> phys={:#x}",
        virt_addr,
        phys
    );

    Ok(PhysAddr::new(phys))
}

/// Remove mapeamento e libera o frame físico
///
/// Versão conveniente que também libera o frame no PMM.
pub unsafe fn unmap_page_and_free(virt_addr: u64, pmm: &mut BitmapFrameAllocator) -> MmResult<()> {
    let phys = unmap_page(virt_addr)?;
    let frame = crate::mm::pmm::PhysFrame::from_start_address(phys);
    pmm.deallocate_frame(frame);
    Ok(())
}
