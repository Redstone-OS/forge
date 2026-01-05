//! # ASLR do Heap
//!
//! Gera offset aleatório para o heap usando TSC + RDRAND.
//!
//! ## Fontes de Entropia
//!
//! 1. **TSC (Time Stamp Counter)**: disponível em todas as CPUs x86
//! 2. **RDRAND**: hardware RNG, se disponível (verificado via CPUID)
//!
//! ## Fallback
//!
//! Se RDRAND não estiver disponível, usa apenas TSC.
//! Isso reduz a entropia mas ainda fornece aleatoriedade razoável.
//!
//! ## Segurança
//!
//! O offset ASLR dificulta ataques que dependem de endereços previsíveis:
//! - Stack spraying
//! - Heap spraying
//! - Return-oriented programming (ROP)
//!
//! ## Debug
//!
//! Em builds debug, ASLR pode ser desabilitado para reprodutibilidade.
//! O seed é registrado para forensics.

use crate::rmm::config::{ASLR_ENTROPY_BITS, ASLR_SLOT_ALIGN};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Flag para desabilitar ASLR em testes
#[cfg(debug_assertions)]
static ASLR_DISABLED: AtomicBool = AtomicBool::new(false);

/// Último seed gerado (para forensics)
static LAST_SEED: AtomicU64 = AtomicU64::new(0);

/// Último offset gerado
static LAST_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Gera offset ASLR para o heap
///
/// Retorna um offset alinhado a `ASLR_SLOT_ALIGN` bytes (2MB por padrão)
/// com `ASLR_ENTROPY_BITS` bits de entropia.
pub fn generate_aslr_offset() -> usize {
    // Verifica se ASLR está desabilitado (debug only)
    #[cfg(debug_assertions)]
    if ASLR_DISABLED.load(Ordering::Relaxed) {
        return 0;
    }

    // Gera entropia
    let entropy = generate_entropy();

    // Salva seed para forensics
    LAST_SEED.store(entropy, Ordering::Relaxed);

    // Calcula offset
    let slots = 1 << ASLR_ENTROPY_BITS; // 256 slots por padrão
    let slot = (entropy as usize) % slots;
    let offset = slot * ASLR_SLOT_ALIGN;

    // Salva offset
    LAST_OFFSET.store(offset as u64, Ordering::Relaxed);

    #[cfg(debug_assertions)]
    crate::kinfo!(
        "(ASLR) seed=0x{:016x}, slot={}, offset=0x{:x}",
        entropy,
        slot,
        offset
    );

    offset
}

/// Gera entropia combinando múltiplas fontes
fn generate_entropy() -> u64 {
    let mut entropy: u64 = 0;

    // Fonte 1: TSC (sempre disponível em x86)
    let tsc = read_tsc();
    entropy ^= tsc;

    // Fonte 2: RDRAND (se disponível)
    if has_rdrand() {
        if let Some(rdrand) = read_rdrand() {
            entropy ^= rdrand;
        }
    }

    // Fonte 3: Mistura adicional usando variações de tempo
    let tsc2 = read_tsc();
    entropy ^= tsc2.wrapping_mul(0x517cc1b727220a95); // Multiplicador de Knuth

    // Fonte 4: Mais uma leitura TSC para variabilidade
    let tsc3 = read_tsc();
    entropy ^= tsc3.rotate_left(23);

    // Mix final
    entropy = mix64(entropy);

    entropy
}

/// Lê Time Stamp Counter (TSC)
#[inline]
fn read_tsc() -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack)
        );
    }

    ((high as u64) << 32) | (low as u64)
}

/// Verifica se RDRAND está disponível via CPUID
fn has_rdrand() -> bool {
    // CPUID.01H:ECX.RDRAND[bit 30]
    let ecx: u32;
    let _ebx: u32; // Precisa salvar ebx para preservar

    unsafe {
        core::arch::asm!(
            "push rbx",
            "mov eax, 1",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            out(reg) _ebx,
            out("ecx") ecx,
            out("eax") _,
            out("edx") _,
            options(nomem, nostack)
        );
    }

    (ecx & (1 << 30)) != 0
}

fn read_rdrand() -> Option<u64> {
    let mut value: u64 = 0;
    let mut success: u8;

    // Tenta até 10 vezes (RDRAND pode falhar temporariamente)
    for _ in 0..10 {
        unsafe {
            core::arch::asm!(
                "rdrand {0:r}",
                "setc {1}",
                out(reg) value,
                out(reg_byte) success,
                options(nomem, nostack)
            );
        }

        if success != 0 {
            return Some(value);
        }
    }

    None
}

/// Função de mistura de 64 bits (SplitMix64)
#[inline]
fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

// =============================================================================
// API de Debug/Forensics
// =============================================================================

/// Retorna o último seed usado
pub fn last_seed() -> u64 {
    LAST_SEED.load(Ordering::Relaxed)
}

/// Retorna o último offset gerado
pub fn last_offset() -> u64 {
    LAST_OFFSET.load(Ordering::Relaxed)
}

/// Desabilita ASLR (apenas em debug builds)
#[cfg(debug_assertions)]
pub fn disable() {
    ASLR_DISABLED.store(true, Ordering::Relaxed);
    crate::kwarn!("(ASLR) DISABLED - for testing only!");
}

/// Habilita ASLR
#[cfg(debug_assertions)]
pub fn enable() {
    ASLR_DISABLED.store(false, Ordering::Relaxed);
    crate::kinfo!("(ASLR) Enabled");
}

/// Verifica se ASLR está habilitado
pub fn is_enabled() -> bool {
    #[cfg(debug_assertions)]
    {
        !ASLR_DISABLED.load(Ordering::Relaxed)
    }

    #[cfg(not(debug_assertions))]
    {
        true
    }
}

/// Gera offset com seed específico (para testes determinísticos)
#[cfg(debug_assertions)]
pub fn generate_with_seed(seed: u64) -> usize {
    LAST_SEED.store(seed, Ordering::Relaxed);

    let entropy = mix64(seed);
    let slots = 1 << ASLR_ENTROPY_BITS;
    let slot = (entropy as usize) % slots;
    let offset = slot * ASLR_SLOT_ALIGN;

    LAST_OFFSET.store(offset as u64, Ordering::Relaxed);

    offset
}

/// Dump informações ASLR
pub fn dump_info() {
    crate::kinfo!("=== ASLR Information ===");
    crate::kinfo!("  Enabled: {}", is_enabled());
    crate::kinfo!("  RDRAND available: {}", has_rdrand());
    crate::kinfo!("  Entropy bits: {}", ASLR_ENTROPY_BITS);
    crate::kinfo!("  Slot alignment: {} KB", ASLR_SLOT_ALIGN / 1024);
    crate::kinfo!("  Last seed: 0x{:016x}", last_seed());
    crate::kinfo!("  Last offset: 0x{:x}", last_offset());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aslr_offset_aligned() {
        let offset = generate_aslr_offset();
        assert_eq!(offset % ASLR_SLOT_ALIGN, 0);
    }

    #[test]
    fn test_aslr_offset_bounded() {
        let offset = generate_aslr_offset();
        let max_offset = (1 << ASLR_ENTROPY_BITS) * ASLR_SLOT_ALIGN;
        assert!(offset < max_offset);
    }

    #[test]
    fn test_mix64() {
        // Verifica que mix64 produz outputs diferentes
        let a = mix64(0);
        let b = mix64(1);
        assert_ne!(a, b);
    }
}
