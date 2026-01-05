//! # Early Boot Allocator
//!
//! Alocador simples usado durante o boot, antes do RMM completo estar disponível.
//! Utiliza a estratégia "bump allocation" que apenas avança um ponteiro.
//!
//! ## Por que existe?
//!
//! O FrameManager precisa de memória para armazenar seus metadados (~64MB para 8GB RAM).
//! Mas quem aloca essa memória se o alocador ainda não existe? (Chicken-and-Egg problem)
//!
//! A solução é usar um alocador primitivo que:
//! 1. Encontra a maior região de memória livre
//! 2. Reserva o que precisar (avançando um ponteiro)
//! 3. Passa o resto do controle para o FrameManager
//!
//! ## Limitações
//!
//! - **Não libera memória** - Bump allocator só avança, nunca volta
//! - **Usado apenas no boot** - Descartado após FrameManager assumir
//! - **Memória perdida** - A região usada não é recuperada (aceitável)
//!
//! ## TODO
//!
//! Investigar recuperação da memória do EarlyAllocator após boot.
//! Possível marcar como reclaimable após todos os metadados serem alocados.

use core::sync::atomic::{AtomicU64, Ordering};

use super::addr::{align_up, PhysAddr};
use super::config::PAGE_SIZE;
use crate::core::boot::handoff::{BootInfo, MemoryMapEntry, MemoryType};

/// Alocador de boot que só avança ponteiro
pub struct EarlyBumpAllocator {
    /// Início da região gerenciada
    start: PhysAddr,

    /// Ponteiro atual (próxima alocação começa aqui)
    current: AtomicU64,

    /// Fim da região gerenciada
    end: PhysAddr,

    /// Total alocado
    allocated: AtomicU64,
}

impl EarlyBumpAllocator {
    /// Cria um novo alocador vazio
    pub const fn new() -> Self {
        Self {
            start: PhysAddr::new(0),
            current: AtomicU64::new(0),
            end: PhysAddr::new(0),
            allocated: AtomicU64::new(0),
        }
    }

    /// Inicializa o alocador com uma região de memória
    ///
    /// # Arguments
    ///
    /// * `start` - Início da região física
    /// * `end` - Fim da região física
    pub fn init(&mut self, start: PhysAddr, end: PhysAddr) {
        self.start = start;
        self.current.store(start.as_u64(), Ordering::SeqCst);
        self.end = end;
        self.allocated.store(0, Ordering::SeqCst);
    }

    /// Aloca uma região física contígua
    ///
    /// # Arguments
    ///
    /// * `size` - Tamanho em bytes
    /// * `align` - Alinhamento (deve ser potência de 2)
    ///
    /// # Returns
    ///
    /// Some(PhysAddr) se sucesso, None se não há memória
    pub fn alloc(&self, size: usize, align: usize) -> Option<PhysAddr> {
        loop {
            let current = self.current.load(Ordering::SeqCst);
            let aligned = align_up(current as usize, align) as u64;
            let new_current = aligned + size as u64;

            if new_current > self.end.as_u64() {
                return None; // OOM
            }

            // Tenta atualizar atomicamente
            if self
                .current
                .compare_exchange(current, new_current, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.allocated.fetch_add(size as u64, Ordering::SeqCst);
                return Some(PhysAddr::new(aligned));
            }
            // Outro CPU alocou, tenta novamente
        }
    }

    /// Aloca N frames (páginas físicas)
    ///
    /// # Arguments
    ///
    /// * `count` - Número de frames
    ///
    /// # Returns
    ///
    /// Some(PhysAddr) do primeiro frame, None se não há memória
    pub fn alloc_frames(&self, count: usize) -> Option<PhysAddr> {
        self.alloc(count * PAGE_SIZE, PAGE_SIZE)
    }

    /// Retorna o total de memória alocada até agora
    pub fn allocated(&self) -> usize {
        self.allocated.load(Ordering::SeqCst) as usize
    }

    /// Retorna a memória disponível restante
    pub fn available(&self) -> usize {
        let current = self.current.load(Ordering::SeqCst);
        (self.end.as_u64() - current) as usize
    }

    /// Retorna a região usada pelo alocador
    ///
    /// Útil para o FrameManager marcar essa região como reservada
    pub fn used_region(&self) -> (PhysAddr, PhysAddr) {
        let current = self.current.load(Ordering::SeqCst);
        (self.start, PhysAddr::new(current))
    }

    /// Retorna a região restante não usada
    ///
    /// Útil para o FrameManager assumir o controle do resto
    pub fn remaining_region(&self) -> (PhysAddr, PhysAddr) {
        let current = self.current.load(Ordering::SeqCst);
        (PhysAddr::new(current), self.end)
    }
}

// =============================================================================
// Instância Global
// =============================================================================

/// Alocador de boot global
static mut EARLY_ALLOCATOR: EarlyBumpAllocator = EarlyBumpAllocator::new();

/// Flag indicando se o early allocator está ativo
static mut EARLY_ACTIVE: bool = false;

/// Inicializa o early allocator com o boot info
///
/// Procura a maior região de memória usável e a reserva para o alocador.
///
/// # Safety
///
/// Deve ser chamado apenas uma vez, durante o boot, antes de qualquer
/// outra alocação.
pub unsafe fn init(boot_info: &'static BootInfo) {
    crate::kinfo!("(RMM/Early) Inicializando early allocator...");

    // Encontra a maior região de memória usável
    let mut best_start: u64 = 0;
    let mut best_size: u64 = 0;

    // Itera sobre o mapa de memória usando os campos do BootInfo
    let entries_ptr = boot_info.memory_map_addr as *const MemoryMapEntry;
    let entries_count = boot_info.memory_map_len as usize;

    for i in 0..entries_count {
        let entry = &*entries_ptr.add(i);

        if entry.typ == MemoryType::Usable && entry.len > best_size {
            // Evita os primeiros 1MB (legacy)
            let start = if entry.base < 0x10_0000 {
                0x10_0000
            } else {
                entry.base
            };

            let size = if start > entry.base {
                entry.len.saturating_sub(start - entry.base)
            } else {
                entry.len
            };

            if size > best_size {
                best_start = start;
                best_size = size;
            }
        }
    }

    if best_size == 0 {
        panic!("(RMM/Early) Nenhuma região de memória usável encontrada!");
    }

    let start = PhysAddr::new(best_start);
    let end = PhysAddr::new(best_start + best_size);

    EARLY_ALLOCATOR.init(start, end);
    EARLY_ACTIVE = true;

    crate::kinfo!(
        "(RMM/Early) Região: 0x{:x} - 0x{:x} ({} MB)",
        best_start,
        best_start + best_size,
        best_size / 1024 / 1024
    );
}

/// Aloca memória do early allocator
///
/// # Panics
///
/// Panic se o early allocator não estiver inicializado
pub fn alloc(size: usize, align: usize) -> Option<PhysAddr> {
    unsafe {
        if !EARLY_ACTIVE {
            panic!("(RMM/Early) Allocator não inicializado!");
        }
        EARLY_ALLOCATOR.alloc(size, align)
    }
}

/// Aloca frames do early allocator
pub fn alloc_frames(count: usize) -> Option<PhysAddr> {
    unsafe {
        if !EARLY_ACTIVE {
            panic!("(RMM/Early) Allocator não inicializado!");
        }
        EARLY_ALLOCATOR.alloc_frames(count)
    }
}

/// Retorna a região usada pelo early allocator
pub fn used_region() -> (PhysAddr, PhysAddr) {
    unsafe { EARLY_ALLOCATOR.used_region() }
}

/// Retorna a região restante
pub fn remaining_region() -> (PhysAddr, PhysAddr) {
    unsafe { EARLY_ALLOCATOR.remaining_region() }
}

/// Desativa o early allocator (após FrameManager assumir)
///
/// # Safety
///
/// Deve ser chamado apenas pelo FrameManager após inicialização
pub unsafe fn deactivate() {
    if EARLY_ACTIVE {
        let allocated = EARLY_ALLOCATOR.allocated();
        crate::kinfo!(
            "(RMM/Early) Desativado. Total alocado: {} KB",
            allocated / 1024
        );
        EARLY_ACTIVE = false;
    }
}

/// Verifica se o early allocator está ativo
pub fn is_active() -> bool {
    unsafe { EARLY_ACTIVE }
}
