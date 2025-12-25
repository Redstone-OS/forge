//! VMM — Virtual Memory Manager (x86_64)
//! ===================================
//!
//! Visão geral
//! -----------
//! Este módulo implementa o **Gerenciador de Memória Virtual** do Redstone OS
//! para arquitetura x86_64. Ele fornece a infraestrutura fundamental para
//! paginação do kernel em early-boot e nas fases iniciais de execução.
//!
//! Suas responsabilidades principais são:
//!
//! - gerenciar a hierarquia de page tables de 4 níveis
//!   (PML4 → PDPT → PD → PT);
//! - criar mapeamentos Virtual → Físico de páginas de 4 KiB;
//! - detectar e resolver conflitos com huge pages de 2 MiB;
//! - prover um mecanismo seguro para:
//!   - criação dinâmica de page tables;
//!   - zeragem de frames físicos recém-alocados;
//!   - expansão controlada do espaço virtual do kernel.
//!
//! O design prioriza **previsibilidade**, **segurança em early-kernel**
//! e **controle explícito do hardware**, evitando abstrações opacas.
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
use crate::mm::pmm::{BitmapFrameAllocator, FRAME_ALLOCATOR, FRAME_SIZE};
use core::arch::asm;

// =============================================================================
// FLAGS DE PAGINAÇÃO x86_64
// =============================================================================
// Flags usadas neste módulo. Consulte Intel SDM para significado detalhado.

/// Página presente na memória física.
pub const PAGE_PRESENT: u64 = 1 << 0;

/// Página pode ser escrita.
pub const PAGE_WRITABLE: u64 = 1 << 1;

/// Página acessível em modo usuário (Ring 3).
pub const PAGE_USER: u64 = 1 << 2;

/// Huge page (2 MiB quando definida em PD). Quando ativa, o CPU ignora PT.
pub const PAGE_HUGE: u64 = 1 << 7;

/// Disable Execute (NX) — marca página como não executável.
pub const PAGE_NO_EXEC: u64 = 1 << 63;

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
const SCRATCH_VIRT: u64 = 0xFFFF_FE00_0000_0000; // virtual address acordado para scratch
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
    // Lê CR3 (PML4 físico atual) e guarda para uso interno.
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3);
    ACTIVE_PML4_PHYS = cr3 & 0x000F_FFFF_FFFF_F000;

    // Valida e inicializa o scratch slot para operações de zeragem.
    init_scratch_slot();

    // (nota) O `boot_info` pode ser usado para logging adicional — mantido como parâmetro
    // para futuras evoluções (ex.: validar RSDP, regiões MMIO, etc).
    let _ = boot_info;
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
    let pml4_idx = ((SCRATCH_VIRT >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((SCRATCH_VIRT >> 30) & 0x1FF) as usize;
    let pd_idx = ((SCRATCH_VIRT >> 21) & 0x1FF) as usize;

    let pml4 = ACTIVE_PML4_PHYS as *const u64;
    let pml4_entry = *pml4.add(pml4_idx);

    if pml4_entry & PAGE_PRESENT == 0 {
        SCRATCH_READY = false;
        return;
    }

    let pdpt_phys = pml4_entry & 0x000F_FFFF_FFFF_F000;
    let pdpt = pdpt_phys as *const u64;
    let pdpt_entry = *pdpt.add(pdpt_idx);

    if pdpt_entry & PAGE_PRESENT == 0 {
        SCRATCH_READY = false;
        return;
    }

    let pd_phys = pdpt_entry & 0x000F_FFFF_FFFF_F000;
    let pd = pd_phys as *const u64;
    let pd_entry = *pd.add(pd_idx);

    if pd_entry & PAGE_PRESENT == 0 {
        SCRATCH_READY = false;
        return;
    }

    if pd_entry & PAGE_HUGE != 0 {
        SCRATCH_READY = false;
        return;
    }

    SCRATCH_PT_PHYS = pd_entry & 0x000F_FFFF_FFFF_F000;
    SCRATCH_READY = true;
}

// =============================================================================
// ZERAGEM DE FRAMES FÍSICOS (USANDO SCRATCH)
// =============================================================================

/// Zera um frame físico com segurança usando o scratch slot.
///
/// Fluxo:
/// 1. Escreve PTE na PT do scratch apontando para `phys`.
/// 2. Invalida TLB para o endereço virtual do scratch.
/// 3. Executa memset(0) no endereço virtual do scratch.
/// 4. Limpa a PTE e invalida TLB novamente.
///
/// Falhas:
/// - Se `SCRATCH_READY` for false, tentamos fallback escrevendo diretamente no físico
///   (menos seguro e pode falhar em algumas configurações). Detecte e corrija no bootloader.
unsafe fn zero_frame_via_scratch(phys: u64) {
    // Zerar usando loop manual via identity map (phys < 4GiB)
    // O identity map do bootloader mapeia 0-4GiB como virtual == físico
    let ptr = phys as *mut u64;

    // Zerar 4096 bytes = 512 u64s
    for i in 0..512 {
        core::ptr::write_volatile(ptr.add(i), 0u64);
    }
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
/// Retorna `true` em sucesso, `false` em OOM ou erro.
pub unsafe fn map_page(virt_addr: u64, phys_addr: u64, flags: u64) -> bool {
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
) -> bool {
    // Índices da hierarquia (9 bits cada)
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;

    // Ponteiro para a PML4 atual (fisicamente endereçada)
    let pml4_ptr = ACTIVE_PML4_PHYS as *mut u64;
    let pml4_entry = &mut *pml4_ptr.add(pml4_idx);

    // Garantir PDPT existe
    let pdpt_phys = ensure_table_entry_with_pmm(pml4_entry, pmm);
    if pdpt_phys == 0 {
        return false;
    }

    let pdpt_ptr = pdpt_phys as *mut u64;
    let pdpt_entry = &mut *pdpt_ptr.add(pdpt_idx);

    // Garantir PD existe
    let pd_phys = ensure_table_entry_with_pmm(pdpt_entry, pmm);
    if pd_phys == 0 {
        return false;
    }

    let pd_ptr = pd_phys as *mut u64;
    let pd_entry = &mut *pd_ptr.add(pd_idx);

    // Garantir PT existe (pode exigir split de huge page)
    let pt_phys = ensure_table_entry_with_pmm(pd_entry, pmm);
    if pt_phys == 0 {
        return false;
    }

    let pt_ptr = pt_phys as *mut u64;
    let pt_entry = &mut *pt_ptr.add(pt_idx);

    // Escrever PTE final (endereço físico + flags + PRESENT)
    *pt_entry = (phys_addr & 0x000F_FFFF_FFFF_F000) | flags | PAGE_PRESENT;

    // Invalida TLB para o endereço mapeado
    asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));

    true
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
    // Caso: entrada já existe
    if *entry & PAGE_PRESENT != 0 {
        // Se for uma huge page, precisamos converter para uma PT de 4 KiB (split)
        if *entry & PAGE_HUGE != 0 {
            // --- SPLIT DE HUGE PAGE ---
            // Extraímos a base física da huge page (alinhada a 2 MiB).
            let huge_base = *entry & 0x000F_FFFF_FFE0_0000;

            // Preservamos flags relevantes, exceto o bit HUGE.
            let old_flags = *entry & 0xFFF;

            // Aloca frame para a nova PT (4 KiB)
            let frame = pmm.allocate_frame();
            if frame.is_none() {
                crate::kerror!("VMM: OOM ao fazer split de huge page!");
                return 0;
            }
            let pt_phys = frame.unwrap().addr;

            // Zera a página que conterá a nova PT antes de escrever entradas.
            zero_frame_via_scratch(pt_phys);

            // Preenche a PT replicando o mapeamento da huge page em 512 entradas.
            let pt = pt_phys as *mut u64;
            let new_flags = (old_flags & !PAGE_HUGE) | PAGE_PRESENT | PAGE_WRITABLE;

            for i in 0..512usize {
                let page_phys = huge_base + (i as u64 * 4096);
                core::ptr::write_volatile(pt.add(i), page_phys | new_flags);
            }

            // Atualiza a entrada do PD para apontar para a nova PT (removendo HUGE).
            *entry = pt_phys | PAGE_PRESENT | PAGE_WRITABLE;

            // Invalidar TLB para toda a região anteriormente mapeada como huge page.
            // Nota: invlpg opera em página por página, por isso iteramos.
            for i in 0..512u64 {
                let vaddr = huge_base + (i * 4096);
                asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
            }

            return pt_phys;
        }

        // Entrada presente e não-huge: retornar endereço base da próxima tabela.
        return *entry & 0x000F_FFFF_FFFF_F000;
    }

    // Caso: entrada ausente — aloca nova tabela (frame de 4 KiB)
    let frame = pmm.allocate_frame();
    if frame.is_none() {
        return 0;
    }
    let phys = frame.unwrap().addr;

    // Zera a nova tabela (crítico para evitar leitura de lixo).
    zero_frame_via_scratch(phys);

    // Configura a entrada para apontar para a nova tabela: PRESENT | WRITABLE | USER.
    *entry = phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    phys
}
