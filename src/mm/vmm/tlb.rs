//! # TLB Management e Shootdown
//!
//! Este módulo implementa o gerenciamento de TLB (Translation Lookaside Buffer)
//! para o kernel Redstone OS, incluindo invalidação local e remota (via IPI).
//!
//! ## 🎯 Propósito
//!
//! O TLB é um cache de traduções de endereços virtuais para físicos.
//! Quando alteramos page tables, precisamos invalidar entradas do TLB para
//! garantir que o CPU use as traduções atualizadas.
//!
//! ## 🏗️ Arquitetura
//!
//! - **Invalidação Local**: `invlpg` para uma página, reload CR3 para flush completo
//! - **TLB Shootdown**: Em SMP, precisamos invalidar o TLB de TODOS os cores
//!   via IPI (Inter-Processor Interrupt)
//!
//! ## ⚠️ CRÍTICO para SMP
//!
//! Sem TLB shootdown, alterações em page tables feitas por um core podem
//! não ser visíveis para outros cores, causando:
//! - Acessos a memória incorreta
//! - Corrupção de dados
//! - Crashes difíceis de diagnosticar
//!
//! ## 🔧 Uso
//!
//! ```rust
//! // Invalidação local (single-core ou quando você sabe que só um core usa)
//! unsafe { tlb::invlpg(VirtAddr::new(0x1000)); }
//!
//! // Flush completo local
//! unsafe { tlb::flush_tlb_local(); }
//!
//! // Shootdown para SMP (invalida em TODOS os cores)
//! #[cfg(feature = "smp")]
//! unsafe { tlb::shootdown::tlb_shootdown(VirtAddr::new(0x1000)); }
//! ```

use crate::mm::addr::VirtAddr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// =============================================================================
// CONFIGURAÇÃO
// =============================================================================

/// Número máximo de CPUs suportadas para TLB shootdown
pub const MAX_CPUS: usize = crate::mm::config::MAX_CPUS;

/// Vetor de IPI usado para TLB shootdown
/// Deve corresponder ao vetor configurado no APIC/IDT
pub const IPI_VECTOR_TLB: u8 = 0xFE;

/// Timeout para aguardar ACKs de shootdown (em iterações do spin loop)
pub const SHOOTDOWN_TIMEOUT: u64 = 10_000_000;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Epoch para sincronização de TLB shootdown
/// Incrementado a cada shootdown para evitar races
static TLB_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Flags de acknowledgment por CPU
/// Cada CPU seta sua flag após processar o IPI
static TLB_ACK_FLAGS: [AtomicBool; MAX_CPUS] = {
    const FALSE: AtomicBool = AtomicBool::new(false);
    [FALSE; MAX_CPUS]
};

/// Endereço pendente para invalidação (0 = flush all)
static PENDING_VADDR: AtomicU64 = AtomicU64::new(0);

/// Flag indicando se shootdown está em progresso
static SHOOTDOWN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// =============================================================================
// ESTATÍSTICAS
// =============================================================================

/// Estatísticas de TLB para telemetria
pub struct TlbStats {
    pub local_invalidations: AtomicU64,
    pub full_flushes: AtomicU64,
    pub shootdowns: AtomicU64,
    pub shootdown_timeouts: AtomicU64,
}

impl TlbStats {
    pub const fn new() -> Self {
        Self {
            local_invalidations: AtomicU64::new(0),
            full_flushes: AtomicU64::new(0),
            shootdowns: AtomicU64::new(0),
            shootdown_timeouts: AtomicU64::new(0),
        }
    }
}

/// Estatísticas globais de TLB
pub static TLB_STATS: TlbStats = TlbStats::new();

// =============================================================================
// INVALIDAÇÃO LOCAL
// =============================================================================

/// Invalida uma única página no TLB local
///
/// Usa a instrução `invlpg` do x86_64.
///
/// # Safety
///
/// Esta função é segura para chamar a qualquer momento, mas o caller
/// deve garantir que a invalidação é necessária e que as page tables
/// foram atualizadas antes de chamar.
///
/// # Exemplo
///
/// ```rust
/// // Após alterar uma PTE
/// unsafe { invlpg(VirtAddr::new(modified_vaddr)); }
/// ```
#[inline(always)]
pub unsafe fn invlpg(vaddr: VirtAddr) {
    core::arch::asm!(
        "invlpg [{}]",
        in(reg) vaddr.as_u64(),
        options(nostack, preserves_flags)
    );
    
    TLB_STATS.local_invalidations.fetch_add(1, Ordering::Relaxed);
}

/// Flush completo do TLB local
///
/// Recarrega CR3, invalidando todas as entradas não-globais do TLB.
///
/// # Safety
///
/// Seguro para chamar, mas é uma operação cara. Use apenas quando
/// necessário (ex: após muitas alterações em page tables).
///
/// # Performance
///
/// Flush completo é significativamente mais lento que `invlpg` individual
/// para poucas páginas. Regra geral: use flush_all se invalidar > 32 páginas.
#[inline(always)]
pub unsafe fn flush_tlb_local() {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
    core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));
    
    TLB_STATS.full_flushes.fetch_add(1, Ordering::Relaxed);
}

/// Invalida range de páginas localmente
///
/// Para ranges pequenos, usa invlpg individual.
/// Para ranges grandes, usa flush completo.
///
/// # Safety
///
/// Mesmas considerações de `invlpg`.
pub unsafe fn invalidate_range_local(start: VirtAddr, num_pages: usize) {
    const THRESHOLD: usize = 32;
    
    if num_pages > THRESHOLD {
        flush_tlb_local();
    } else {
        let mut addr = start.as_u64();
        for _ in 0..num_pages {
            invlpg(VirtAddr::new(addr));
            addr += 4096;
        }
    }
}

// =============================================================================
// TLB SHOOTDOWN (SMP)
// =============================================================================

/// Módulo de TLB Shootdown para sistemas multicore
///
/// Só é compilado quando a feature `smp` está habilitada.
#[cfg(feature = "smp")]
pub mod shootdown {
    use super::*;
    
    /// Handler de IPI para TLB shootdown
    ///
    /// Esta função é chamada pelo interrupt handler quando recebe
    /// o IPI de TLB shootdown. Deve ser registrada no IDT.
    ///
    /// # Safety
    ///
    /// Deve ser chamada apenas no contexto de interrupt handler.
    pub fn handle_tlb_ipi() {
        // Verificar se há shootdown em progresso
        if !SHOOTDOWN_IN_PROGRESS.load(Ordering::Acquire) {
            return;
        }
        
        // Obter endereço pendente
        let vaddr = PENDING_VADDR.load(Ordering::Acquire);
        
        // Invalidar
        if vaddr == 0 {
            // Flush completo solicitado
            unsafe { flush_tlb_local(); }
        } else {
            // Invalidação específica
            unsafe { invlpg(VirtAddr::new(vaddr)); }
        }
        
        // Marcar ACK
        let cpu_id = get_cpu_id();
        if cpu_id < MAX_CPUS {
            TLB_ACK_FLAGS[cpu_id].store(true, Ordering::Release);
        }
    }
    
    /// Invalida endereço em TODOS os CPUs
    ///
    /// Este é o ponto de entrada principal para TLB shootdown.
    /// Invalida o endereço localmente e envia IPI para todos os outros cores.
    ///
    /// # Safety
    ///
    /// - O caller deve garantir que as page tables foram atualizadas
    /// - Não deve ser chamada em contexto de interrupt
    ///
    /// # Blocking
    ///
    /// Esta função bloqueia até que todos os cores confirmem a invalidação
    /// ou até timeout (para evitar deadlock se um core travou).
    pub unsafe fn tlb_shootdown(vaddr: VirtAddr) {
        let current_cpu = get_cpu_id();
        let num_cpus = get_num_cpus();
        
        // Se só temos 1 CPU, faz invalidação local e retorna
        if num_cpus <= 1 {
            invlpg(vaddr);
            return;
        }
        
        // Incrementar epoch
        let _epoch = TLB_EPOCH.fetch_add(1, Ordering::SeqCst);
        
        // Resetar flags de ACK
        for i in 0..num_cpus {
            TLB_ACK_FLAGS[i].store(false, Ordering::Release);
        }
        
        // Setar endereço pendente
        PENDING_VADDR.store(vaddr.as_u64(), Ordering::Release);
        
        // Marcar shootdown em progresso
        SHOOTDOWN_IN_PROGRESS.store(true, Ordering::Release);
        
        // Invalidar localmente primeiro
        invlpg(vaddr);
        TLB_ACK_FLAGS[current_cpu].store(true, Ordering::Release);
        
        // Enviar IPI para outros cores
        for cpu in 0..num_cpus {
            if cpu != current_cpu {
                send_ipi(cpu, IPI_VECTOR_TLB);
            }
        }
        
        // Spin wait para ACKs (com timeout)
        let mut timeout = SHOOTDOWN_TIMEOUT;
        loop {
            let mut all_acked = true;
            for cpu in 0..num_cpus {
                if !TLB_ACK_FLAGS[cpu].load(Ordering::Acquire) {
                    all_acked = false;
                    break;
                }
            }
            
            if all_acked {
                break;
            }
            
            timeout -= 1;
            if timeout == 0 {
                crate::kwarn!("(TLB) Shootdown timeout! Alguns cores não responderam.");
                TLB_STATS.shootdown_timeouts.fetch_add(1, Ordering::Relaxed);
                break;
            }
            
            core::hint::spin_loop();
        }
        
        // Limpar estado
        SHOOTDOWN_IN_PROGRESS.store(false, Ordering::Release);
        PENDING_VADDR.store(0, Ordering::Release);
        
        TLB_STATS.shootdowns.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Flush completo em TODOS os CPUs
    ///
    /// Similar a `tlb_shootdown`, mas faz flush completo em vez de
    /// invalidar um endereço específico.
    pub unsafe fn tlb_shootdown_all() {
        let current_cpu = get_cpu_id();
        let num_cpus = get_num_cpus();
        
        if num_cpus <= 1 {
            flush_tlb_local();
            return;
        }
        
        let _epoch = TLB_EPOCH.fetch_add(1, Ordering::SeqCst);
        
        for i in 0..num_cpus {
            TLB_ACK_FLAGS[i].store(false, Ordering::Release);
        }
        
        // 0 = flush all
        PENDING_VADDR.store(0, Ordering::Release);
        SHOOTDOWN_IN_PROGRESS.store(true, Ordering::Release);
        
        flush_tlb_local();
        TLB_ACK_FLAGS[current_cpu].store(true, Ordering::Release);
        
        for cpu in 0..num_cpus {
            if cpu != current_cpu {
                send_ipi(cpu, IPI_VECTOR_TLB);
            }
        }
        
        let mut timeout = SHOOTDOWN_TIMEOUT;
        loop {
            let mut all_acked = true;
            for cpu in 0..num_cpus {
                if !TLB_ACK_FLAGS[cpu].load(Ordering::Acquire) {
                    all_acked = false;
                    break;
                }
            }
            
            if all_acked { break; }
            
            timeout -= 1;
            if timeout == 0 {
                crate::kwarn!("(TLB) Shootdown ALL timeout!");
                TLB_STATS.shootdown_timeouts.fetch_add(1, Ordering::Relaxed);
                break;
            }
            
            core::hint::spin_loop();
        }
        
        SHOOTDOWN_IN_PROGRESS.store(false, Ordering::Release);
        TLB_STATS.shootdowns.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Invalida range de páginas em todos os CPUs
    pub unsafe fn tlb_shootdown_range(start: VirtAddr, num_pages: usize) {
        const THRESHOLD: usize = 8;
        
        if num_pages > THRESHOLD {
            // Muitas páginas - flush all é mais eficiente
            tlb_shootdown_all();
        } else {
            // Poucas páginas - invalidar individualmente
            // Nota: Isso é simplificado. Em produção, poderia batch IPIs.
            let mut addr = start.as_u64();
            for _ in 0..num_pages {
                tlb_shootdown(VirtAddr::new(addr));
                addr += 4096;
            }
        }
    }
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Obtém o ID do CPU atual
///
/// TODO: Implementar corretamente via APIC ID ou CPUID
fn get_cpu_id() -> usize {
    // Placeholder - deve ser implementado em arch/
    // Por enquanto retorna 0 (BSP)
    0
}

/// Obtém o número de CPUs ativas
///
/// TODO: Implementar corretamente via contagem de APs inicializados
fn get_num_cpus() -> usize {
    // Placeholder - deve ser implementado em arch/
    // Por enquanto retorna 1 (single core)
    1
}

/// Envia IPI para um CPU específico
///
/// TODO: Implementar via APIC
#[cfg(feature = "smp")]
fn send_ipi(target_cpu: usize, vector: u8) {
    // Placeholder - deve ser implementado em arch/apic
    let _ = (target_cpu, vector);
    crate::ktrace!("(TLB) send_ipi para CPU {} (placeholder)", target_cpu);
}

// =============================================================================
// API UNIFICADA
// =============================================================================

/// Invalida uma página, usando shootdown se SMP está habilitado
///
/// Esta é a função de alto nível que automaticamente escolhe entre
/// invalidação local e shootdown baseado na configuração de build.
///
/// # Safety
///
/// O caller deve garantir que as page tables foram atualizadas.
pub unsafe fn invalidate_page(vaddr: VirtAddr) {
    #[cfg(feature = "smp")]
    {
        shootdown::tlb_shootdown(vaddr);
    }
    
    #[cfg(not(feature = "smp"))]
    {
        invlpg(vaddr);
    }
}

/// Flush completo, usando shootdown se SMP está habilitado
pub unsafe fn invalidate_all() {
    #[cfg(feature = "smp")]
    {
        shootdown::tlb_shootdown_all();
    }
    
    #[cfg(not(feature = "smp"))]
    {
        flush_tlb_local();
    }
}

// =============================================================================
// DEBUG E TELEMETRIA
// =============================================================================

/// Imprime estatísticas de TLB
pub fn print_tlb_stats() {
    crate::kinfo!("╔══════════════════════════════════════╗");
    crate::kinfo!("║       ESTATÍSTICAS DE TLB            ║");
    crate::kinfo!("╚══════════════════════════════════════╝");
    crate::kinfo!("  Invalidações locais: {}", 
        TLB_STATS.local_invalidations.load(Ordering::Relaxed));
    crate::kinfo!("  Flushes completos: {}", 
        TLB_STATS.full_flushes.load(Ordering::Relaxed));
    crate::kinfo!("  Shootdowns SMP: {}", 
        TLB_STATS.shootdowns.load(Ordering::Relaxed));
    crate::kinfo!("  Shootdown timeouts: {}", 
        TLB_STATS.shootdown_timeouts.load(Ordering::Relaxed));
}
