//! Testes de conversão de endereços (phys_to_virt, virt_to_phys)

use crate::klib::test_framework::{TestCase, TestResult};
use crate::mm::addr::{self, PhysAddr};

/// Testes de endereços
pub const ADDR_TESTS: &[TestCase] = &[
    TestCase::new("addr_phys_accessible", test_phys_accessible),
    TestCase::new("addr_round_trip", test_round_trip),
    TestCase::new("addr_frame_align", test_frame_align),
];

/// Teste: verificar se endereço físico é acessível
fn test_phys_accessible() -> TestResult {
    let test_phys_val: u64 = 0x1000000; // 16 MB
    let test_phys = PhysAddr::new(test_phys_val);

    if !addr::is_phys_accessible(test_phys) {
        crate::kerror!("(Addr) Phys deveria ser acessível=", test_phys_val);
        return TestResult::Fail;
    }

    crate::ktrace!("(Addr) is_phys_accessible OK");
    TestResult::Pass
}

/// Teste: phys -> virt -> phys round-trip
fn test_round_trip() -> TestResult {
    let test_phys_val: u64 = 0x1000000; // 16 MB
    let test_phys = PhysAddr::new(test_phys_val);

    let virt = addr::phys_to_virt(test_phys);
    let back = match addr::virt_to_phys(virt) {
        Some(p) => p,
        None => {
            crate::kerror!("(Addr) virt_to_phys falhou");
            return TestResult::Fail;
        }
    };

    if test_phys != back {
        crate::kerror!("(Addr) Round-trip falhou! back=", back.as_u64());
        return TestResult::Fail;
    }

    crate::ktrace!("(Addr) Round-trip OK");
    TestResult::Pass
}

/// Teste: alinhamento de frames
fn test_frame_align() -> TestResult {
    let test_addr: u64 = 0x12345678;
    let aligned = crate::mm::config::align_down(test_addr as usize, 4096) as u64;
    let expected: u64 = 0x12345000;

    if aligned != expected {
        crate::kerror!("(Addr) Alinhamento errado! aligned=", aligned);
        return TestResult::Fail;
    }

    crate::ktrace!("(Addr) Alinhamento de frame OK");
    TestResult::Pass
}
