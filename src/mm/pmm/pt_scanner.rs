//! Page Table Scanner
//!
//! Este módulo escaneia a hierarquia de page tables a partir do CR3
//! e marca todos os frames usados como ocupados no PMM.
//!
//! # Por que isso é necessário?
//!
//! O bootloader (Ignite) aloca page tables em memória marcada como "Usable"
//! no memory map. Se o PMM não souber que esses frames estão em uso,
//! ele pode alocá-los para outras estruturas, corrompendo as page tables
//! ativas e causando crashes.
//!
//! # Uso
//!
//! ```rust
//! unsafe {
//!     pt_scanner::mark_bootloader_page_tables(&mut pmm);
//! }
//! ```

use crate::mm::addr::phys_to_virt;
use crate::mm::config::{PAGE_HUGE, PAGE_MASK, PAGE_PRESENT};
use crate::mm::pmm::BitmapFrameAllocator;
use core::arch::asm;

/// Número máximo de page tables a escanear (proteção contra loops infinitos)
const MAX_TABLES_TO_SCAN: usize = 16384;

/// Estatísticas de escaneamento por nível
pub struct ScanStats {
    pub pml4_frames: usize,
    pub pdpt_frames: usize,
    pub pd_frames: usize,
    pub pt_frames: usize,
    pub data_frames: usize,
    pub already_marked: usize,
}

impl ScanStats {
    pub const fn new() -> Self {
        Self {
            pml4_frames: 0,
            pdpt_frames: 0,
            pd_frames: 0,
            pt_frames: 0,
            data_frames: 0,
            already_marked: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.pml4_frames + self.pdpt_frames + self.pd_frames + self.pt_frames + self.data_frames
    }
}

/// Estatísticas globais da última execução
static mut SCAN_STATS: ScanStats = ScanStats::new();

/// Escaneia a hierarquia de page tables a partir do CR3 atual
/// e marca todos os frames ocupados no PMM.
///
/// # Safety
///
/// - Deve ser chamado apenas durante early-boot, antes de liberar frames no PMM
/// - O CR3 deve conter page tables válidas do bootloader
/// - O PMM deve estar parcialmente inicializado (bitmap alocado)
pub unsafe fn mark_bootloader_page_tables(pmm: &mut BitmapFrameAllocator) {
    crate::ktrace!("(PTScanner) [1] Entrando na funcao...");

    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));

    crate::ktrace!("(PTScanner) [2] CR3 lido=", cr3);

    let pml4_phys = cr3 & PAGE_MASK;

    crate::ktrace!("(PTScanner) [3] PML4 phys=", pml4_phys);

    crate::kinfo!(
        "(PTScanner) Escaneando tabelas de página a partir de CR3=",
        pml4_phys
    );

    crate::ktrace!("(PTScanner) [4] Resetando SCAN_STATS...");

    SCAN_STATS = ScanStats::new();

    crate::ktrace!("(PTScanner) [5] Chamando mark_frame para PML4...");

    if mark_frame(pmm, pml4_phys, "PML4") {
        SCAN_STATS.pml4_frames += 1;
    } else {
        SCAN_STATS.already_marked += 1;
    }

    crate::ktrace!("(PTScanner) [6] PML4 marcado, chamando scan_pml4...");

    scan_pml4(pmm, pml4_phys);

    // Log de resumo
    crate::kdebug!("(PTScanner) Resumo: PML4=", SCAN_STATS.pml4_frames as u64);
    crate::kdebug!("(PTScanner)         PDPT=", SCAN_STATS.pdpt_frames as u64);
    crate::kdebug!("(PTScanner)         PD  =", SCAN_STATS.pd_frames as u64);
    crate::kdebug!("(PTScanner)         PT  =", SCAN_STATS.pt_frames as u64);
    crate::kdebug!(
        "(PTScanner) Já marcados  =",
        SCAN_STATS.already_marked as u64
    );

    crate::kinfo!(
        "(PTScanner) Total quadros protegidos=",
        SCAN_STATS.total() as u64
    );
}

/// Marca um frame como ocupado no PMM usando o método real
///
/// Retorna true se o frame foi marcado, false se já estava marcado
///
/// NOTA: Evita macros de formatação (ktrace!) para não gerar SSE/#UD
unsafe fn mark_frame(pmm: &mut BitmapFrameAllocator, phys: u64, _level: &str) -> bool {
    // Calcular índices manualmente para evitar chamar método que pode gerar SSE
    let frame_idx = (phys / 4096) as usize;
    // Verificar se está no range válido
    if frame_idx >= pmm.total_frames() {
        return false;
    }

    if pmm.is_frame_used(frame_idx as u64) {
        return false;
    }

    pmm.mark_frame_used(frame_idx as u64, true);
    true
}

unsafe fn scan_pml4(pmm: &mut BitmapFrameAllocator, pml4_phys: u64) {
    let pml4: *const u64 = phys_to_virt::<u64>(pml4_phys);

    // Usando while manual em vez de for (iterador Range pode gerar #UD)
    let mut i: usize = 0;
    while i < 512 {
        let entry = *pml4.add(i);

        if entry & PAGE_PRESENT != 0 {
            let pdpt_phys = entry & PAGE_MASK;

            if mark_frame(pmm, pdpt_phys, "PDPT") {
                SCAN_STATS.pdpt_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }

            // Scan PDPT
            scan_pdpt(pmm, pdpt_phys);

            if SCAN_STATS.total() >= MAX_TABLES_TO_SCAN {
                crate::kwarn!(
                    "(PTScanner) Limite de tabelas atingido limit=",
                    MAX_TABLES_TO_SCAN as u64
                );
                return;
            }
        }
        i += 1;
    }
}

unsafe fn scan_pdpt(pmm: &mut BitmapFrameAllocator, pdpt_phys: u64) {
    let pdpt: *const u64 = phys_to_virt::<u64>(pdpt_phys);

    let mut i: usize = 0;
    while i < 512 {
        let entry = *pdpt.add(i);

        if entry & PAGE_PRESENT != 0 {
            let phys = entry & PAGE_MASK;
            // Se for huge page (1GB), apenas pular - proteção feita pelo init_free_regions
            if entry & PAGE_HUGE != 0 {
                i += 1;
                continue;
            }

            if mark_frame(pmm, phys, "PD") {
                SCAN_STATS.pd_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }

            // Scan PD
            scan_pd(pmm, phys);

            if SCAN_STATS.total() >= MAX_TABLES_TO_SCAN {
                return;
            }
        }
        i += 1;
    }
}

unsafe fn scan_pd(pmm: &mut BitmapFrameAllocator, pd_phys: u64) {
    let pd: *const u64 = phys_to_virt::<u64>(pd_phys);

    let mut i: usize = 0;
    while i < 512 {
        let entry = *pd.add(i);

        if entry & PAGE_PRESENT != 0 {
            let phys = entry & PAGE_MASK;
            // Se for huge page (2MB), apenas pular - proteção feita pelo init_free_regions
            if entry & PAGE_HUGE != 0 {
                i += 1;
                continue;
            }

            if mark_frame(pmm, phys, "PT") {
                SCAN_STATS.pt_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }

            // Scan PT
            scan_pt(pmm, phys);

            if SCAN_STATS.total() >= MAX_TABLES_TO_SCAN {
                return;
            }
        }
        i += 1;
    }
}

unsafe fn scan_pt(pmm: &mut BitmapFrameAllocator, pt_phys: u64) {
    let pt: *const u64 = phys_to_virt::<u64>(pt_phys);

    let mut i: usize = 0;
    while i < 512 {
        let entry = *pt.add(i);

        if entry & PAGE_PRESENT != 0 {
            let data_phys = entry & PAGE_MASK;
            if mark_frame(pmm, data_phys, "DATA_4K") {
                SCAN_STATS.data_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }
        }
        i += 1;
    }
}

/// Retorna o número total de frames marcados na última execução
pub fn marked_frames_count() -> usize {
    unsafe { SCAN_STATS.total() }
}

/// Retorna estatísticas detalhadas da última execução
pub fn get_stats() -> ScanStats {
    unsafe {
        ScanStats {
            pml4_frames: SCAN_STATS.pml4_frames,
            pdpt_frames: SCAN_STATS.pdpt_frames,
            pd_frames: SCAN_STATS.pd_frames,
            pt_frames: SCAN_STATS.pt_frames,
            data_frames: SCAN_STATS.data_frames,
            already_marked: SCAN_STATS.already_marked,
        }
    }
}
