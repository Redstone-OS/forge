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

use crate::sync::spinlock::Spinlock;
use core::mem::{align_of, size_of};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::addr::{align_up, PhysAddr};
use super::config::{HHDM_BASE, PAGE_SIZE};
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
    ///
    /// # Panics
    ///
    /// Panic se a região for menor que 16MB (mínimo para metadados)
    pub fn init(&mut self, start: PhysAddr, end: PhysAddr) {
        let size = end.as_u64() - start.as_u64();

        // Valida tamanho mínimo (16MB para casos pequenos)
        const MIN_SIZE: u64 = 16 * 1024 * 1024;
        if size < MIN_SIZE {
            panic!(
                "(RMM/Early) Região muito pequena: {} MB (mínimo {} MB)",
                size / 1024 / 1024,
                MIN_SIZE / 1024 / 1024
            );
        }

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
    ///
    /// # Panics
    ///
    /// Panic se align não for potência de 2
    pub fn alloc(&self, size: usize, align: usize) -> Option<PhysAddr> {
        // Valida alinhamento
        if !align.is_power_of_two() {
            panic!("(RMM/Early) Alinhamento inválido: {}", align);
        }

        loop {
            let current = self.current.load(Ordering::SeqCst);
            let aligned = align_up(current as usize, align) as u64;
            let new_current = aligned + size as u64;

            // Verifica overflow
            if new_current > self.end.as_u64() {
                crate::kerror!(
                    "(RMM/Early) OOM! Tentou alocar {} bytes, apenas {} bytes disponíveis",
                    size,
                    self.available()
                );
                return None;
            }

            // Tenta atualizar atomicamente (CAS)
            if self
                .current
                .compare_exchange(current, new_current, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.allocated.fetch_add(size as u64, Ordering::SeqCst);
                return Some(PhysAddr::new(aligned));
            }
            // Outro CPU alocou, tenta novamente (spin)
        }
    }

    /// Aloca N frames (páginas físicas)
    ///
    /// # Arguments
    ///
    /// * `count` - Número de frames (4KB cada)
    ///
    /// # Returns
    ///
    /// Some(PhysAddr) do primeiro frame, None se não há memória
    pub fn alloc_frames(&self, count: usize) -> Option<PhysAddr> {
        self.alloc(count * PAGE_SIZE, PAGE_SIZE)
    }

    /// Aloca e retorna uma slice estática mutável
    ///
    /// **CRÍTICO:** Esta é a função que resolve o problema do Vec!
    ///
    /// # Arguments
    ///
    /// * `count` - Número de elementos no slice
    ///
    /// # Returns
    ///
    /// &'static mut [T] - Slice estática que vive para sempre
    ///
    /// # Safety
    ///
    /// - HHDM deve estar ativo (para converter PhysAddr → VirtAddr)
    /// - Memória retornada é zeroed
    /// - Lifetime 'static é seguro porque essa memória nunca é liberada
    ///
    /// # Panics
    ///
    /// Panic se não conseguir alocar (OOM durante boot é fatal)
    pub fn alloc_slice<T>(&self, count: usize) -> &'static mut [T] {
        // TODO: Revisar
        if count == 0 {
            return &mut [];
        }

        let size = size_of::<T>() * count;
        let align = align_of::<T>();

        let phys = self
            .alloc(size, align)
            .expect("(RMM/Early) OOM ao alocar slice - boot impossível!");

        let virt = phys_to_virt(phys);

        unsafe {
            let ptr = virt as *mut T;

            core::slice::from_raw_parts_mut(ptr, count)
        }
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
// Conversão Física → Virtual (HHDM)
// =============================================================================

/// Converte endereço físico para virtual via HHDM
///
/// # Arguments
///
/// * `phys` - Endereço físico
///
/// # Returns
///
/// Endereço virtual correspondente
///
/// # Panics
///
/// Panic se HHDM não estiver inicializado (bug do kernel)
#[inline]
fn phys_to_virt(phys: PhysAddr) -> u64 {
    // HHDM: mapeamento direto em 0xFFFF_8000_0000_0000
    HHDM_BASE + phys.as_u64()
}

// =============================================================================
// Instância Global (Thread-Safe)
// =============================================================================

/// Alocador de boot global
static EARLY_ALLOCATOR: Spinlock<EarlyBumpAllocator> = Spinlock::new(EarlyBumpAllocator::new());

/// Flag indicando se o early allocator está ativo
static EARLY_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Inicializa o early allocator com o boot info
///
/// Procura a maior região de memória usável e a reserva para o alocador.
///
/// # Safety
///
/// Deve ser chamado apenas uma vez, durante o boot, antes de qualquer
/// outra alocação de memória.
///
/// # Panics
///
/// Panic se nenhuma região de memória usável for encontrada.
pub unsafe fn init(boot_info: &'static BootInfo) {
    crate::kinfo!("(RMM/Early) Inicializando early allocator...");

    // Verifica se já foi inicializado
    if EARLY_ACTIVE.load(Ordering::SeqCst) {
        panic!("(RMM/Early) Já foi inicializado!");
    }

    // Encontra a maior região de memória usável
    let mut best_start: u64 = 0;
    let mut best_size: u64 = 0;

    // Itera sobre o mapa de memória
    let entries_ptr = boot_info.memory_map_addr as *const MemoryMapEntry;
    let entries_count = boot_info.memory_map_len as usize;

    for i in 0..entries_count {
        let entry = &*entries_ptr.add(i);

        if entry.typ == MemoryType::Usable {
            // Evita os primeiros 1MB (legacy ISA, BIOS, etc)
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

            // Procura a maior região
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

    // Inicializa o alocador
    let mut allocator = EARLY_ALLOCATOR.lock();
    allocator.init(start, end);
    drop(allocator); // Libera lock explicitamente

    EARLY_ACTIVE.store(true, Ordering::SeqCst);

    crate::kinfo!(
        "(RMM/Early) Região: 0x{:x} - 0x{:x} ({} MB disponíveis)",
        best_start,
        best_start + best_size,
        best_size / 1024 / 1024
    );
}

/// Aloca memória do early allocator
///
/// # Panics
///
/// Panic se o early allocator não estiver inicializado ou se OOM.
pub fn alloc(size: usize, align: usize) -> PhysAddr {
    if !EARLY_ACTIVE.load(Ordering::SeqCst) {
        panic!("(RMM/Early) Allocator não inicializado!");
    }

    let allocator = EARLY_ALLOCATOR.lock();
    allocator
        .alloc(size, align)
        .expect("(RMM/Early) OOM durante boot!")
}

/// Aloca frames do early allocator
///
/// # Panics
///
/// Panic se o early allocator não estiver inicializado ou se OOM.
pub fn alloc_frames(count: usize) -> PhysAddr {
    if !EARLY_ACTIVE.load(Ordering::SeqCst) {
        panic!("(RMM/Early) Allocator não inicializado!");
    }

    let allocator = EARLY_ALLOCATOR.lock();
    allocator
        .alloc_frames(count)
        .expect("(RMM/Early) OOM durante boot!")
}

/// Aloca slice estático (NOVA FUNÇÃO - RESOLVE O PROBLEMA!)
///
/// # Type Parameters
///
/// * `T` - Tipo do elemento
///
/// # Arguments
///
/// * `count` - Número de elementos
///
/// # Returns
///
/// &'static mut [T] - Slice zeroed
///
/// # Panics
///
/// Panic se não inicializado ou OOM.
///
/// # Example
///
/// ```rust,no_run
/// // Aloca array de 1024 ChunkManagers
/// let chunks: &'static mut [ChunkManager] = early::alloc_slice(1024);
/// ```
pub fn alloc_slice<T>(count: usize) -> &'static mut [T] {
    if !EARLY_ACTIVE.load(Ordering::SeqCst) {
        panic!("(RMM/Early) Allocator não inicializado!");
    }

    let allocator = EARLY_ALLOCATOR.lock();
    allocator.alloc_slice(count)
}

/// Retorna a região usada pelo early allocator
pub fn used_region() -> (PhysAddr, PhysAddr) {
    let allocator = EARLY_ALLOCATOR.lock();
    allocator.used_region()
}

/// Retorna a região restante
pub fn remaining_region() -> (PhysAddr, PhysAddr) {
    let allocator = EARLY_ALLOCATOR.lock();
    allocator.remaining_region()
}

/// Desativa o early allocator (após FrameManager assumir)
///
/// # Safety
///
/// Deve ser chamado apenas pelo FrameManager após inicialização completa.
pub unsafe fn deactivate() {
    if EARLY_ACTIVE.load(Ordering::SeqCst) {
        let allocator = EARLY_ALLOCATOR.lock();
        let allocated = allocator.allocated();
        drop(allocator);

        crate::kinfo!(
            "(RMM/Early) Desativado. Total alocado: {} KB",
            allocated / 1024
        );

        EARLY_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// Verifica se o early allocator está ativo
pub fn is_active() -> bool {
    EARLY_ACTIVE.load(Ordering::SeqCst)
}

// =============================================================================
// Testes (se quiser validar a lógica)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_basic() {
        let mut alloc = EarlyBumpAllocator::new();
        alloc.init(PhysAddr::new(0x1000), PhysAddr::new(0x10000));

        // Aloca 100 bytes
        let addr = alloc.alloc(100, 8).unwrap();
        assert_eq!(addr.as_u64(), 0x1000);

        // Aloca mais 200 bytes
        let addr2 = alloc.alloc(200, 8).unwrap();
        assert!(addr2.as_u64() > addr.as_u64());
    }

    #[test]
    fn test_alloc_alignment() {
        let mut alloc = EarlyBumpAllocator::new();
        alloc.init(PhysAddr::new(0x1001), PhysAddr::new(0x10000));

        // Aloca com alinhamento de 4096
        let addr = alloc.alloc(100, 4096).unwrap();
        assert_eq!(addr.as_u64() % 4096, 0);
    }

    #[test]
    fn test_alloc_oom() {
        let mut alloc = EarlyBumpAllocator::new();
        alloc.init(PhysAddr::new(0x1000), PhysAddr::new(0x2000)); // Apenas 4KB

        // Tenta alocar 8KB (maior que disponível)
        let result = alloc.alloc(8192, 8);
        assert!(result.is_none());
    }
}
