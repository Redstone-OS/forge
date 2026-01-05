//! # TLB Management
//!
//! Gerenciamento de Translation Lookaside Buffer (TLB) e IPI shootdown para SMP.
//!
//! ## TLB Flush
//!
//! O TLB cacheia traduções de endereços virtuais para físicos.
//! Quando mapeamentos mudam, o TLB precisa ser invalidado.
//!
//! ## Single CPU vs SMP
//!
//! - **Single CPU**: `invlpg` local é suficiente
//! - **SMP**: Precisa enviar IPI para outras CPUs invalidarem seus TLBs
//!
//! ## Otimizações
//!
//! - Pages GLOBAL não são flushed no switch de CR3
//! - Batch flush é mais eficiente para ranges grandes
//! - Full flush (reload CR3) para mudanças extensas

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::PAGE_SIZE;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Threshold para fazer full flush ao invés de invlpg individual
const FULL_FLUSH_THRESHOLD: usize = 32;

/// Contador de TLB flushes (para debug)
static TLB_FLUSH_COUNT: AtomicUsize = AtomicUsize::new(0);
static TLB_SHOOTDOWN_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Flag indicando se SMP está ativo
static SMP_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Ativa modo SMP para TLB shootdowns
pub fn enable_smp() {
    SMP_ACTIVE.store(true, Ordering::Release);
}

/// Desativa modo SMP
pub fn disable_smp() {
    SMP_ACTIVE.store(false, Ordering::Release);
}

/// Verifica se SMP está ativo
#[inline]
pub fn is_smp_active() -> bool {
    SMP_ACTIVE.load(Ordering::Acquire)
}

// =============================================================================
// Flush Local
// =============================================================================

/// Flush TLB para um endereço específico (local)
///
/// Usa instrução `invlpg` para invalidar apenas uma entrada.
#[inline]
pub fn flush_tlb(virt: VirtAddr) {
    TLB_FLUSH_COUNT.fetch_add(1, Ordering::Relaxed);

    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) virt.as_u64(),
            options(nostack, preserves_flags)
        );
    }

    // Se SMP ativo, fazer shootdown
    if is_smp_active() {
        shootdown_single(virt);
    }
}

/// Flush TLB completo (local)
///
/// Recarrega CR3 para invalidar todas as entradas não-global.
#[inline]
pub fn flush_tlb_all() {
    TLB_FLUSH_COUNT.fetch_add(1, Ordering::Relaxed);

    unsafe {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }

    // Se SMP ativo, fazer shootdown em todas as CPUs
    if is_smp_active() {
        shootdown_all();
    }
}

/// Flush TLB incluindo páginas globais (local)
///
/// Desabilita e reabilita PGE bit em CR4 para flush completo.
pub fn flush_tlb_global() {
    TLB_FLUSH_COUNT.fetch_add(1, Ordering::Relaxed);

    unsafe {
        // Ler CR4
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));

        // Toggle PGE bit (bit 7)
        let cr4_no_pge = cr4 & !(1 << 7);
        core::arch::asm!("mov cr4, {}", in(reg) cr4_no_pge, options(nostack));
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack));
    }

    if is_smp_active() {
        shootdown_all();
    }
}

/// Flush range de páginas (local)
///
/// Para ranges pequenos, usa invlpg individual.
/// Para ranges grandes, faz full flush.
pub fn flush_tlb_range(start: VirtAddr, end: VirtAddr) {
    let page_count = ((end.as_u64() - start.as_u64()) as usize + PAGE_SIZE - 1) / PAGE_SIZE;

    if page_count >= FULL_FLUSH_THRESHOLD {
        flush_tlb_all();
    } else {
        let mut addr = start;
        while addr < end {
            unsafe {
                core::arch::asm!(
                    "invlpg [{}]",
                    in(reg) addr.as_u64(),
                    options(nostack, preserves_flags)
                );
            }
            addr = addr + PAGE_SIZE as u64;
        }
        TLB_FLUSH_COUNT.fetch_add(page_count, Ordering::Relaxed);

        if is_smp_active() {
            shootdown_range(start, end);
        }
    }
}

// =============================================================================
// IPI Shootdown
// =============================================================================

/// Dados para IPI shootdown
#[repr(C)]
pub struct ShootdownRequest {
    /// Tipo de flush
    pub kind: ShootdownKind,
    /// Endereço inicial (para Single e Range)
    pub start: u64,
    /// Endereço final (para Range)
    pub end: u64,
    /// Contador de ACKs recebidos
    pub ack_count: AtomicUsize,
    /// Número de CPUs que devem responder
    pub target_count: usize,
}

/// Tipo de shootdown
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShootdownKind {
    /// Flush de uma página
    Single,
    /// Flush de range
    Range,
    /// Flush completo
    All,
}

/// Request de shootdown pendente (global para simplificar)
static mut SHOOTDOWN_REQUEST: Option<ShootdownRequest> = None;

/// Lock para shootdown
static SHOOTDOWN_LOCK: AtomicBool = AtomicBool::new(false);

/// Envia IPI para TLB shootdown de uma página
fn shootdown_single(virt: VirtAddr) {
    TLB_SHOOTDOWN_COUNT.fetch_add(1, Ordering::Relaxed);

    // Adquire lock
    while SHOOTDOWN_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }

    // Configura request
    let cpu_count = get_online_cpu_count();
    if cpu_count <= 1 {
        SHOOTDOWN_LOCK.store(false, Ordering::Release);
        return;
    }

    unsafe {
        SHOOTDOWN_REQUEST = Some(ShootdownRequest {
            kind: ShootdownKind::Single,
            start: virt.as_u64(),
            end: 0,
            ack_count: AtomicUsize::new(0),
            target_count: cpu_count - 1, // Excluding self
        });
    }

    // Envia IPI para outras CPUs
    send_tlb_ipi();

    // Aguarda ACKs
    wait_for_shootdown_acks();

    // Limpa request
    unsafe {
        SHOOTDOWN_REQUEST = None;
    }

    SHOOTDOWN_LOCK.store(false, Ordering::Release);
}

/// Envia IPI para TLB shootdown de range
fn shootdown_range(start: VirtAddr, end: VirtAddr) {
    TLB_SHOOTDOWN_COUNT.fetch_add(1, Ordering::Relaxed);

    while SHOOTDOWN_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }

    let cpu_count = get_online_cpu_count();
    if cpu_count <= 1 {
        SHOOTDOWN_LOCK.store(false, Ordering::Release);
        return;
    }

    unsafe {
        SHOOTDOWN_REQUEST = Some(ShootdownRequest {
            kind: ShootdownKind::Range,
            start: start.as_u64(),
            end: end.as_u64(),
            ack_count: AtomicUsize::new(0),
            target_count: cpu_count - 1,
        });
    }

    send_tlb_ipi();
    wait_for_shootdown_acks();

    unsafe {
        SHOOTDOWN_REQUEST = None;
    }

    SHOOTDOWN_LOCK.store(false, Ordering::Release);
}

/// Envia IPI para TLB shootdown completo
fn shootdown_all() {
    TLB_SHOOTDOWN_COUNT.fetch_add(1, Ordering::Relaxed);

    while SHOOTDOWN_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }

    let cpu_count = get_online_cpu_count();
    if cpu_count <= 1 {
        SHOOTDOWN_LOCK.store(false, Ordering::Release);
        return;
    }

    unsafe {
        SHOOTDOWN_REQUEST = Some(ShootdownRequest {
            kind: ShootdownKind::All,
            start: 0,
            end: 0,
            ack_count: AtomicUsize::new(0),
            target_count: cpu_count - 1,
        });
    }

    send_tlb_ipi();
    wait_for_shootdown_acks();

    unsafe {
        SHOOTDOWN_REQUEST = None;
    }

    SHOOTDOWN_LOCK.store(false, Ordering::Release);
}

/// Handler de IPI de TLB shootdown (chamado em cada CPU)
///
/// Deve ser chamado pelo handler de interrupção de IPI.
pub fn handle_tlb_ipi() {
    let request = unsafe { SHOOTDOWN_REQUEST.as_ref() };

    if let Some(req) = request {
        match req.kind {
            ShootdownKind::Single => {
                let virt = VirtAddr::new(req.start);
                unsafe {
                    core::arch::asm!(
                        "invlpg [{}]",
                        in(reg) virt.as_u64(),
                        options(nostack, preserves_flags)
                    );
                }
            }
            ShootdownKind::Range => {
                let start = VirtAddr::new(req.start);
                let end = VirtAddr::new(req.end);
                let page_count =
                    ((end.as_u64() - start.as_u64()) as usize + PAGE_SIZE - 1) / PAGE_SIZE;

                if page_count >= FULL_FLUSH_THRESHOLD {
                    unsafe {
                        let cr3: u64;
                        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
                        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
                    }
                } else {
                    let mut addr = start;
                    while addr < end {
                        unsafe {
                            core::arch::asm!(
                                "invlpg [{}]",
                                in(reg) addr.as_u64(),
                                options(nostack, preserves_flags)
                            );
                        }
                        addr = addr + PAGE_SIZE as u64;
                    }
                }
            }
            ShootdownKind::All => unsafe {
                let cr3: u64;
                core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
                core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
            },
        }

        // Envia ACK
        req.ack_count.fetch_add(1, Ordering::Release);
    }
}

/// Aguarda ACKs de todas as CPUs
fn wait_for_shootdown_acks() {
    let request = unsafe { SHOOTDOWN_REQUEST.as_ref() };

    if let Some(req) = request {
        // Spin wait com timeout implícito (em produção, adicionar timeout real)
        let mut iterations = 0u64;
        while req.ack_count.load(Ordering::Acquire) < req.target_count {
            core::hint::spin_loop();
            iterations += 1;

            if iterations > 1_000_000_000 {
                // Timeout! Log e continue
                crate::kwarn!("(TLB) Shootdown timeout!");
                break;
            }
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Retorna número de CPUs online
fn get_online_cpu_count() -> usize {
    // TODO: Integrar com módulo SMP
    1
}

/// Envia IPI de TLB para todas as outras CPUs
fn send_tlb_ipi() {
    // TODO: Integrar com APIC para enviar IPI
    // Por enquanto, no-op (single CPU mode)
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Retorna contagem de TLB flushes
pub fn flush_count() -> usize {
    TLB_FLUSH_COUNT.load(Ordering::Relaxed)
}

/// Retorna contagem de shootdowns
pub fn shootdown_count() -> usize {
    TLB_SHOOTDOWN_COUNT.load(Ordering::Relaxed)
}

/// Reseta contadores
pub fn reset_stats() {
    TLB_FLUSH_COUNT.store(0, Ordering::Relaxed);
    TLB_SHOOTDOWN_COUNT.store(0, Ordering::Relaxed);
}
