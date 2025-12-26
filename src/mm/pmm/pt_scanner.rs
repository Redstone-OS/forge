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

use crate::mm::addr::{phys_to_virt, PhysAddr};
use crate::mm::config::{PAGE_HUGE, PAGE_MASK, PAGE_PRESENT};
use crate::mm::pmm::BitmapFrameAllocator;
use core::arch::asm;

/// Número máximo de page tables a escanear (proteção contra loops infinitos)
const MAX_TABLES_TO_SCAN: usize = 1024;

/// Estatísticas de escaneamento por nível
pub struct ScanStats {
    pub pml4_frames: usize,
    pub pdpt_frames: usize,
    pub pd_frames: usize,
    pub pt_frames: usize,
    pub already_marked: usize,
}

impl ScanStats {
    pub const fn new() -> Self {
        Self {
            pml4_frames: 0,
            pdpt_frames: 0,
            pd_frames: 0,
            pt_frames: 0,
            already_marked: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.pml4_frames + self.pdpt_frames + self.pd_frames + self.pt_frames
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
    crate::drivers::serial::write_str_raw("[PTS] A: entrada\r\n");

    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));

    crate::drivers::serial::write_str_raw("[PTS] B: pos-cr3\r\n");

    let pml4_phys = cr3 & PAGE_MASK;

    crate::drivers::serial::write_str_raw("[PTS] C: pre-kinfo\r\n");

    crate::kinfo!(
        "(PTScanner) Escaneando page tables a partir de CR3={:#x}",
        pml4_phys
    );

    crate::drivers::serial::write_str_raw("[PTS] D: pos-kinfo\r\n");

    // Resetar estatísticas
    crate::drivers::serial::write_str_raw("[PTS] E: pre-stats-reset\r\n");
    SCAN_STATS = ScanStats::new();
    crate::drivers::serial::write_str_raw("[PTS] F: pos-stats-reset\r\n");

    // Marcar a própria PML4
    crate::drivers::serial::write_str_raw("[PTS] G: pre-mark-pml4\r\n");
    if mark_frame(pmm, pml4_phys, "PML4") {
        SCAN_STATS.pml4_frames += 1;
    } else {
        SCAN_STATS.already_marked += 1;
    }
    crate::drivers::serial::write_str_raw("[PTS] H: pos-mark-pml4\r\n");

    // Escanear hierarquia
    crate::drivers::serial::write_str_raw("[PTS] I: pre-scan-pml4\r\n");
    scan_pml4(pmm, pml4_phys);

    // Log de resumo
    crate::kinfo!(
        "(PTScanner) Resumo: PML4={}, PDPT={}, PD={}, PT={} (já marcados: {})",
        SCAN_STATS.pml4_frames,
        SCAN_STATS.pdpt_frames,
        SCAN_STATS.pd_frames,
        SCAN_STATS.pt_frames,
        SCAN_STATS.already_marked
    );

    crate::kinfo!(
        "(PTScanner) Total: {} frames de page tables protegidos",
        SCAN_STATS.total()
    );
}

/// Marca um frame como ocupado no PMM usando o método real
///
/// Retorna true se o frame foi marcado, false se já estava marcado
///
/// NOTA: Evita macros de formatação (ktrace!) para não gerar SSE/#UD
unsafe fn mark_frame(pmm: &mut BitmapFrameAllocator, phys: u64, _level: &str) -> bool {
    crate::drivers::serial::write_str_raw("[MF] A: is_frame_used\r\n");

    // Verificar se já está marcado como ocupado
    if pmm.is_frame_used(phys) {
        crate::drivers::serial::write_str_raw("[MF] B: already used\r\n");
        return false;
    }

    crate::drivers::serial::write_str_raw("[MF] C: mark_frame_used\r\n");

    // Marcar como ocupado usando o método do PMM
    let marked = pmm.mark_frame_used(phys);

    crate::drivers::serial::write_str_raw("[MF] D: done\r\n");

    marked
}

/// Escaneia a PML4 e suas tabelas filhas
unsafe fn scan_pml4(pmm: &mut BitmapFrameAllocator, pml4_phys: u64) {
    let pml4: *const u64 = phys_to_virt(PhysAddr::new(pml4_phys)).as_ptr();

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
                    "(PTScanner) Limite de {} tabelas atingido",
                    MAX_TABLES_TO_SCAN
                );
                return;
            }
        }
        i += 1;
    }
}

/// Escaneia uma PDPT e suas tabelas filhas
unsafe fn scan_pdpt(pmm: &mut BitmapFrameAllocator, pdpt_phys: u64) {
    let pdpt: *const u64 = phys_to_virt(PhysAddr::new(pdpt_phys)).as_ptr();

    let mut i: usize = 0;
    while i < 512 {
        let entry = *pdpt.add(i);

        if entry & PAGE_PRESENT != 0 {
            // Se for huge page (1GB), não tem PD abaixo
            if entry & PAGE_HUGE != 0 {
                i += 1;
                continue;
            }

            let pd_phys = entry & PAGE_MASK;

            if mark_frame(pmm, pd_phys, "PD") {
                SCAN_STATS.pd_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }

            // Scan PD
            scan_pd(pmm, pd_phys);

            if SCAN_STATS.total() >= MAX_TABLES_TO_SCAN {
                return;
            }
        }
        i += 1;
    }
}

/// Escaneia um PD e suas tabelas filhas
unsafe fn scan_pd(pmm: &mut BitmapFrameAllocator, pd_phys: u64) {
    let pd: *const u64 = phys_to_virt(PhysAddr::new(pd_phys)).as_ptr();

    let mut i: usize = 0;
    while i < 512 {
        let entry = *pd.add(i);

        if entry & PAGE_PRESENT != 0 {
            // Se for huge page (2MB), não tem PT abaixo
            if entry & PAGE_HUGE != 0 {
                i += 1;
                continue;
            }

            let pt_phys = entry & PAGE_MASK;

            if mark_frame(pmm, pt_phys, "PT") {
                SCAN_STATS.pt_frames += 1;
            } else {
                SCAN_STATS.already_marked += 1;
            }

            // Não precisamos escanear dentro da PT,
            // pois as PTEs apontam para frames de dados, não page tables

            if SCAN_STATS.total() >= MAX_TABLES_TO_SCAN {
                return;
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
            already_marked: SCAN_STATS.already_marked,
        }
    }
}
