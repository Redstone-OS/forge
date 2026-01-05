//! # Page Scanner
//!
//! Scanner de páginas para coleta de informações e decisão de reclaim.
//!
//! ## Funcionamento
//!
//! O scanner percorre páginas e coleta informações:
//! - Bit A (Accessed) - página foi acessada
//! - Bit D (Dirty) - página foi modificada
//! - Refcount - quantos PTEs apontam para a página
//! - Owner - tipo de página (anon, file, kernel)
//!
//! ## Scanning Modes
//!
//! - **Full**: Escaneia todas as páginas (lento)
//! - **Incremental**: Escaneia um batch por vez
//! - **Targeted**: Escaneia apenas páginas de um tipo

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::frame::{FrameFlags, FrameInfo, FrameOwner};

use alloc::vec::Vec;

// =============================================================================
// ScanResult
// =============================================================================

/// Resultado de scan de uma página
#[derive(Debug, Clone, Copy)]
pub struct ScanResult {
    /// Endereço físico
    pub phys: PhysAddr,
    /// Página foi acessada
    pub accessed: bool,
    /// Página foi modificada
    pub dirty: bool,
    /// Número de mapeamentos
    pub map_count: u32,
    /// Pode ser evictada
    pub evictable: bool,
    /// Pode ser swapped
    pub swappable: bool,
    /// Prioridade de eviction (maior = evictar primeiro)
    pub priority: u32,
}

impl ScanResult {
    /// Página limpa e não acessada recentemente (boa para eviction)
    pub fn is_cold_clean(&self) -> bool {
        !self.accessed && !self.dirty
    }

    /// Página quente (acessada recentemente)
    pub fn is_hot(&self) -> bool {
        self.accessed
    }
}

// =============================================================================
// ScanConfig
// =============================================================================

/// Configuração do scanner
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Número máximo de páginas para escanear
    pub max_pages: usize,
    /// Incluir páginas anônimas
    pub scan_anon: bool,
    /// Incluir páginas file-backed
    pub scan_file: bool,
    /// Limpar bit accessed após ler
    pub clear_accessed: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_pages: 256,
            scan_anon: true,
            scan_file: true,
            clear_accessed: true,
        }
    }
}

// =============================================================================
// PageScanner
// =============================================================================

/// Scanner de páginas
pub struct PageScanner {
    /// Configuração
    config: ScanConfig,
    /// Posição atual no scan
    current_pos: usize,
    /// Total de páginas escaneadas
    scanned: u64,
    /// Páginas evictáveis encontradas
    evictable_found: u64,
}

impl PageScanner {
    /// Cria novo scanner
    pub fn new(config: ScanConfig) -> Self {
        Self {
            config,
            current_pos: 0,
            scanned: 0,
            evictable_found: 0,
        }
    }

    /// Escaneia batch de páginas da lista
    pub fn scan_batch(&mut self, pages: &[PhysAddr]) -> Vec<ScanResult> {
        let mut results = Vec::with_capacity(self.config.max_pages);

        for &phys in pages.iter().take(self.config.max_pages) {
            if let Some(result) = self.scan_page(phys) {
                if result.evictable {
                    self.evictable_found += 1;
                }
                results.push(result);
            }
            self.scanned += 1;
        }

        // Ordena por prioridade (maior primeiro)
        results.sort_by(|a, b| b.priority.cmp(&a.priority));

        results
    }

    /// Escaneia uma página
    fn scan_page(&mut self, phys: PhysAddr) -> Option<ScanResult> {
        // TODO: Obter FrameInfo da página
        // Por enquanto, retorna resultado simulado

        let accessed = false; // Ler de FrameInfo
        let dirty = false; // Ler de FrameInfo
        let map_count = 1; // Ler de FrameInfo rmap

        // Calcula prioridade
        let mut priority = 0u32;

        // Páginas não acessadas têm maior prioridade
        if !accessed {
            priority += 100;
        }

        // Páginas limpas têm maior prioridade (sem I/O para evict)
        if !dirty {
            priority += 50;
        }

        // Menos mapeamentos = menor custo de TLB shootdown
        priority += 10u32.saturating_sub(map_count);

        Some(ScanResult {
            phys,
            accessed,
            dirty,
            map_count,
            evictable: !dirty || self.config.scan_anon,
            swappable: self.config.scan_anon,
            priority,
        })
    }

    /// Reseta posição do scanner
    pub fn reset(&mut self) {
        self.current_pos = 0;
    }

    /// Estatísticas
    pub fn stats(&self) -> (u64, u64) {
        (self.scanned, self.evictable_found)
    }
}

impl Default for PageScanner {
    fn default() -> Self {
        Self::new(ScanConfig::default())
    }
}

// =============================================================================
// Refcount Aging
// =============================================================================

/// Implementa aging baseado em refcount
///
/// Páginas com muitos acessos recentes têm idade menor.
/// Idade aumenta quando página não é acessada.
pub struct RefcountAging {
    /// Fator de decaimento (shift right)
    decay_shift: u32,
    /// Bônus por acesso
    access_bonus: u32,
}

impl RefcountAging {
    pub fn new() -> Self {
        Self {
            decay_shift: 1,   // Divide idade por 2
            access_bonus: 64, // Bônus por acesso
        }
    }

    /// Calcula nova idade após decay
    pub fn decay(&self, age: u32) -> u32 {
        age >> self.decay_shift
    }

    /// Calcula nova idade após acesso
    pub fn access(&self, age: u32) -> u32 {
        age.saturating_add(self.access_bonus)
    }

    /// Verifica se página é candidata para eviction
    pub fn is_evict_candidate(&self, age: u32, threshold: u32) -> bool {
        age < threshold
    }
}

impl Default for RefcountAging {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Working Set Estimation
// =============================================================================

/// Estimativa do working set de um processo
#[derive(Debug, Default)]
pub struct WorkingSetEstimator {
    /// Número de páginas no working set
    size: usize,
    /// Janela de tempo para estimativa (em scans)
    window: usize,
    /// Histórico de acessos
    history: alloc::collections::VecDeque<usize>,
}

impl WorkingSetEstimator {
    /// Cria novo estimador
    pub fn new(window: usize) -> Self {
        Self {
            size: 0,
            window,
            history: alloc::collections::VecDeque::with_capacity(window),
        }
    }

    /// Atualiza com novo scan
    pub fn update(&mut self, accessed_count: usize) {
        // Adiciona ao histórico
        if self.history.len() >= self.window {
            self.history.pop_front();
        }
        self.history.push_back(accessed_count);

        // Calcula média móvel
        self.size = self.history.iter().sum::<usize>() / self.history.len().max(1);
    }

    /// Retorna tamanho estimado do working set
    pub fn size(&self) -> usize {
        self.size
    }

    /// Retorna em bytes
    pub fn size_bytes(&self) -> usize {
        self.size * PAGE_SIZE
    }
}
