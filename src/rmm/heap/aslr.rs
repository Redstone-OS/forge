//! # ASLR do Heap
//!
//! Gera offset aleatório para o heap usando TSC + RDRAND.
//!
//! ## Fontes de Entropia
//!
//! 1. TSC (Time Stamp Counter): disponível em todas as CPUs x86
//! 2. RDRAND: hardware RNG, se disponível
//!
//! ## Fallback
//!
//! Se RDRAND não estiver disponível, usa apenas TSC.
//! Isso reduz a entropia mas ainda fornece aleatoriedade razoável.
//!
//! ## TODO: Forensics
//!
//! - Registrar seed gerado para debug/forensics
//! - Permitir desabilitar ASLR via flag de debug para testes determinísticos
//! - Verificar CPUID antes de usar RDRAND

use crate::rmm::config::{ASLR_ENTROPY_BITS, ASLR_SLOT_ALIGN};

/// Flag para desabilitar ASLR em testes
#[cfg(debug_assertions)]
pub static mut ASLR_DISABLED: bool = false;

/// Último seed gerado (para forensics)
static mut LAST_SEED: u64 = 0;

/// Gera offset ASLR para o heap
pub fn generate_aslr_offset() -> usize {
    let mut entropy: u64 = 0;

    // Fonte 1: TSC (timing)
    entropy ^= read_tsc();

    // Fonte 2: RDRAND (se disponível)
    if let Some(rand) = try_rdrand() {
        entropy ^= rand;
    }

    // Mix simples
    entropy = mix(entropy);

    // Extrai bits de entropia
    let slots = 1 << ASLR_ENTROPY_BITS;
    let slot = (entropy as usize) % slots;

    slot * ASLR_SLOT_ALIGN
}

/// Lê TSC (Time Stamp Counter)
#[inline]
fn read_tsc() -> u64 {
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack)
        );
        ((hi as u64) << 32) | (lo as u64)
    }
}

/// Tenta RDRAND
fn try_rdrand() -> Option<u64> {
    // TODO: Verificar CPUID para RDRAND support
    let mut val: u64;
    let ok: u8;
    unsafe {
        core::arch::asm!(
            "rdrand {}",
            "setc {}",
            out(reg) val,
            out(reg_byte) ok,
            options(nomem, nostack)
        );
    }
    if ok != 0 {
        Some(val)
    } else {
        None
    }
}

/// Mix de entropia (FNV-1a like)
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}
