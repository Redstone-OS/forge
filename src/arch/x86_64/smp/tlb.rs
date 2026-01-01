//! # TLB Shootdown (x86_64 SMP)
//!
//! Invalidação de TLB em sistemas multicore via IPI batching.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Número máximo de endereços para batch
pub const MAX_BATCH_SIZE: usize = 32;

/// Request de TLB shootdown pendente
pub struct TlbShootdownRequest {
    /// Endereços a invalidar
    pub addresses: [u64; MAX_BATCH_SIZE],
    /// Número de endereços
    pub count: usize,
    /// CR3/PCID alvo
    pub target_cr3: u64,
    /// Máscara de CPUs que precisam responder
    pub cpu_mask: u64,
    /// Contagem de CPUs que responderam
    pub ack_count: AtomicU64,
    /// Request está ativo
    pub active: AtomicBool,
}

impl TlbShootdownRequest {
    pub const fn new() -> Self {
        Self {
            addresses: [0; MAX_BATCH_SIZE],
            count: 0,
            target_cr3: 0,
            cpu_mask: 0,
            ack_count: AtomicU64::new(0),
            active: AtomicBool::new(false),
        }
    }
}

/// Invalida uma página em todas as CPUs
pub fn invalidate_page(addr: u64) {
    // Fast path: invalidar localmente
    invalidate_local(addr);

    // TODO: Enviar IPI para outras CPUs quando SMP estiver ativo
}

/// Invalida range de páginas
pub fn invalidate_range(start: u64, end: u64) {
    let mut addr = start;
    while addr < end {
        invalidate_page(addr);
        addr += crate::mm::config::PAGE_SIZE as u64;
    }
}

/// Invalidação completa de TLB
pub fn flush_all() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        core::arch::asm!("mov cr3, {}", in(reg) cr3);
    }
}

/// Invalida entrada local de TLB
#[inline]
pub fn invalidate_local(addr: u64) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

/// Handler de IPI para TLB shootdown
pub fn handle_ipi() {
    // Será chamado pelo handler de interrupção
    // TODO: Implementar quando SMP estiver ativo
}

/// Batch de invalidações pendentes (per-CPU)
pub struct TlbBatch {
    addresses: [u64; MAX_BATCH_SIZE],
    count: usize,
}

impl TlbBatch {
    pub const fn new() -> Self {
        Self {
            addresses: [0; MAX_BATCH_SIZE],
            count: 0,
        }
    }

    pub fn add(&mut self, addr: u64) {
        if self.count < MAX_BATCH_SIZE {
            self.addresses[self.count] = addr;
            self.count += 1;
        } else {
            self.flush();
            self.addresses[0] = addr;
            self.count = 1;
        }
    }

    pub fn flush(&mut self) {
        for i in 0..self.count {
            invalidate_page(self.addresses[i]);
        }
        self.count = 0;
    }
}
