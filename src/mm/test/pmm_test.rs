//! Testes do PMM (Physical Memory Manager)

use crate::klib::test_framework::{TestCase, TestResult};
use crate::mm::config::PAGE_SIZE;
use crate::mm::pmm;

/// Testes do PMM
pub const PMM_TESTS: &[TestCase] = &[
    TestCase::new("pmm_alloc_dealloc", test_alloc_dealloc),
    TestCase::new("pmm_frame_alignment", test_frame_alignment),
];

/// Teste básico: alocar e desalocar frames
fn test_alloc_dealloc() -> TestResult {
    let mut pmm = pmm::FRAME_ALLOCATOR.lock();
    let mut frames = [0u64; 10];

    // Alocar 10 frames
    let mut i = 0usize;
    while i < 10 {
        let frame = pmm.allocate_frame();
        if frame.is_none() {
            crate::kerror!("(PMM) OOM ao alocar frame índice=", i as u64);
            return TestResult::Fail;
        }
        frames[i] = frame.unwrap().addr();
        i += 1;
    }

    crate::ktrace!("(PMM) 10 frames alocados OK");

    // Desalocar
    let mut j = 0usize;
    while j < 10 {
        use crate::mm::addr::PhysAddr;
        use crate::mm::pmm::PhysFrame;
        let frame = PhysFrame::from_start_address(PhysAddr::new(frames[j]));
        pmm.deallocate_frame(frame);
        j += 1;
    }

    crate::ktrace!("(PMM) 10 frames desalocados OK");
    TestResult::Pass
}

/// Teste: verificar alinhamento dos frames
fn test_frame_alignment() -> TestResult {
    let mut pmm = pmm::FRAME_ALLOCATOR.lock();

    // Alocar alguns frames e verificar alinhamento
    let mut i = 0;
    while i < 5 {
        if let Some(frame) = pmm.allocate_frame() {
            let addr = frame.addr();
            if addr % PAGE_SIZE as u64 != 0 {
                crate::kerror!("(PMM) Frame desalinhado em=", addr);
                return TestResult::Fail;
            }
            // Desalocar imediatamente
            use crate::mm::addr::PhysAddr;
            use crate::mm::pmm::PhysFrame;
            pmm.deallocate_frame(PhysFrame::from_start_address(PhysAddr::new(addr)));
        }
        i += 1;
    }

    TestResult::Pass
}
