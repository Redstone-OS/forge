//! # Virtual Memory Area (VMA)
//!
//! Define regiões de memória virtual e suas propriedades.
//!
//! ## Propriedades de uma VMA
//!
//! - **start/end**: Range de endereços virtuais
//! - **protection**: Permissões (R/W/X)
//! - **flags**: Características (shared, CoW, growable)
//! - **intent**: Semântica (code, data, heap, stack, mmap)
//!
//! ## Uso
//!
//! VMAs são usadas para:
//! - Rastrear regiões mapeadas de um processo
//! - Determinar como tratar page faults
//! - Implementar proteção de memória
//! - Suportar mmap, brk, execve

use crate::rmm::addr::VirtAddr;
use crate::rmm::virt::mapper::MapFlags;

// =============================================================================
// MemoryIntent
// =============================================================================

/// Intenção semântica da região de memória
///
/// Define como a região será usada, afetando comportamento de page faults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIntent {
    /// Código executável (.text)
    /// - Somente leitura + execução
    /// - Pode ser compartilhado entre processos
    Code,

    /// Dados inicializados (.data)
    /// - Leitura/escrita
    /// - CoW em fork
    Data,

    /// Dados não inicializados (.bss)
    /// - Leitura/escrita
    /// - Lazy zeroing
    Bss,

    /// Dados somente leitura (.rodata)
    /// - Somente leitura
    /// - Compartilhável
    Rodata,

    /// Heap (brk/sbrk)
    /// - Leitura/escrita
    /// - Cresce para cima
    Heap,

    /// Stack
    /// - Leitura/escrita
    /// - Cresce para baixo
    /// - Guard page abaixo
    Stack,

    /// mmap anônimo
    /// - Permissões variáveis
    /// - Lazy allocation
    AnonMmap,

    /// mmap de arquivo
    /// - Pode ser shared ou private
    /// - Page cache backed
    FileMmap,

    /// Memória compartilhada IPC
    /// - Múltiplos processos
    /// - Sem CoW
    SharedMemory,

    /// Buffer de dispositivo
    /// - MMIO, framebuffer
    /// - No cache
    DeviceBuffer,

    /// Guard page
    /// - Não acessível
    /// - Detecta overflow
    Guard,

    /// Região reservada
    /// - Endereço reservado mas não mapeado
    Reserved,
}

impl MemoryIntent {
    /// Proteção padrão para esta intenção
    pub fn default_protection(&self) -> Protection {
        match self {
            Self::Code => Protection::RX,
            Self::Data | Self::Bss | Self::Heap | Self::Stack => Protection::RW,
            Self::Rodata => Protection::RO,
            Self::AnonMmap | Self::FileMmap | Self::SharedMemory => Protection::RW,
            Self::DeviceBuffer => Protection::RW,
            Self::Guard | Self::Reserved => Protection::NONE,
        }
    }

    /// Esta região pode crescer?
    pub fn is_growable(&self) -> bool {
        matches!(self, Self::Heap | Self::Stack)
    }

    /// Esta região pode ser compartilhada?
    pub fn is_shareable(&self) -> bool {
        matches!(self, Self::Code | Self::Rodata | Self::SharedMemory)
    }

    /// Esta região deve usar CoW em fork?
    pub fn uses_cow(&self) -> bool {
        matches!(
            self,
            Self::Data | Self::Bss | Self::Heap | Self::Stack | Self::AnonMmap
        )
    }

    /// Esta região é lazy allocated?
    pub fn is_lazy(&self) -> bool {
        matches!(self, Self::Bss | Self::AnonMmap | Self::Stack)
    }
}

// =============================================================================
// Protection
// =============================================================================

/// Proteção de memória (permissões)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Protection(u8);

impl Protection {
    const READ_BIT: u8 = 1 << 0;
    const WRITE_BIT: u8 = 1 << 1;
    const EXEC_BIT: u8 = 1 << 2;

    /// Sem acesso
    pub const NONE: Self = Self(0);
    /// Somente leitura
    pub const RO: Self = Self(Self::READ_BIT);
    /// Leitura + escrita
    pub const RW: Self = Self(Self::READ_BIT | Self::WRITE_BIT);
    /// Leitura + execução
    pub const RX: Self = Self(Self::READ_BIT | Self::EXEC_BIT);
    /// Leitura + escrita + execução
    pub const RWX: Self = Self(Self::READ_BIT | Self::WRITE_BIT | Self::EXEC_BIT);
    /// Escrita (raro)
    pub const WO: Self = Self(Self::WRITE_BIT);

    /// Cria proteção customizada
    pub const fn new(read: bool, write: bool, exec: bool) -> Self {
        let mut bits = 0;
        if read {
            bits |= Self::READ_BIT;
        }
        if write {
            bits |= Self::WRITE_BIT;
        }
        if exec {
            bits |= Self::EXEC_BIT;
        }
        Self(bits)
    }

    /// Pode ler?
    #[inline]
    pub const fn can_read(&self) -> bool {
        (self.0 & Self::READ_BIT) != 0
    }

    /// Pode escrever?
    #[inline]
    pub const fn can_write(&self) -> bool {
        (self.0 & Self::WRITE_BIT) != 0
    }

    /// Pode executar?
    #[inline]
    pub const fn can_exec(&self) -> bool {
        (self.0 & Self::EXEC_BIT) != 0
    }

    /// União de proteções
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Interseção de proteções
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Remove proteção
    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Converte para MapFlags
    pub fn to_map_flags(&self) -> MapFlags {
        let mut flags = MapFlags::PRESENT.union(MapFlags::USER);

        if self.can_write() {
            flags = flags.union(MapFlags::WRITABLE);
        }

        if !self.can_exec() {
            flags = flags.union(MapFlags::NO_EXEC);
        }

        flags
    }

    /// Cria de mprotect flags (compatível com Linux)
    pub fn from_prot_flags(prot: u32) -> Self {
        let read = (prot & 1) != 0;
        let write = (prot & 2) != 0;
        let exec = (prot & 4) != 0;
        Self::new(read, write, exec)
    }

    /// Converte para prot flags (compatível com Linux)
    pub fn to_prot_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.can_read() {
            flags |= 1;
        }
        if self.can_write() {
            flags |= 2;
        }
        if self.can_exec() {
            flags |= 4;
        }
        flags
    }
}

// =============================================================================
// VmaFlags
// =============================================================================

/// Flags adicionais de VMA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmaFlags(u32);

impl VmaFlags {
    /// Nenhuma flag
    pub const NONE: Self = Self(0);
    /// Região pode crescer
    pub const GROWABLE: Self = Self(1 << 0);
    /// Região está locked (não pode ser swapped)
    pub const LOCKED: Self = Self(1 << 1);
    /// Região é compartilhada (mmap MAP_SHARED)
    pub const SHARED: Self = Self(1 << 2);
    /// Região usa Copy-on-Write
    pub const COW: Self = Self(1 << 3);
    /// Stack (cresce para baixo)
    pub const STACK: Self = Self(1 << 4);
    /// Populate on map (não lazy)
    pub const POPULATE: Self = Self(1 << 5);
    /// Don't reserve swap
    pub const NORESERVE: Self = Self(1 << 6);
    /// Huge pages preferred
    pub const HUGETLB: Self = Self(1 << 7);
    /// Região tem backing de arquivo
    pub const FILE_BACKED: Self = Self(1 << 8);
    /// Guard região (não acessível)
    pub const GUARD: Self = Self(1 << 9);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn bits(&self) -> u32 {
        self.0
    }
}

// =============================================================================
// VMA
// =============================================================================

/// Virtual Memory Area
///
/// Representa uma região contígua de memória virtual com mesmas propriedades.
#[derive(Clone)]
pub struct Vma {
    /// Endereço inicial (alinhado a página)
    pub start: VirtAddr,
    /// Endereço final (exclusivo, alinhado a página)
    pub end: VirtAddr,
    /// Proteção de memória
    pub protection: Protection,
    /// Flags adicionais
    pub flags: VmaFlags,
    /// Intenção semântica
    pub intent: MemoryIntent,
    /// Offset em arquivo (para file-backed)
    pub file_offset: u64,
    /// ID do arquivo (para file-backed)
    pub file_id: u64,
}

impl Vma {
    /// Cria nova VMA
    pub fn new(start: VirtAddr, size: usize, prot: Protection, intent: MemoryIntent) -> Self {
        let end = start + size as u64;
        let mut flags = VmaFlags::NONE;

        // Adiciona flags baseado na intenção
        if intent.is_growable() {
            flags = flags.union(VmaFlags::GROWABLE);
        }
        if intent == MemoryIntent::Stack {
            flags = flags.union(VmaFlags::STACK);
        }
        if intent.uses_cow() {
            // CoW será ativado no fork
        }
        if intent == MemoryIntent::SharedMemory {
            flags = flags.union(VmaFlags::SHARED);
        }

        Self {
            start,
            end,
            protection: prot,
            flags,
            intent,
            file_offset: 0,
            file_id: 0,
        }
    }

    /// Cria VMA file-backed
    pub fn new_file(
        start: VirtAddr,
        size: usize,
        prot: Protection,
        file_id: u64,
        offset: u64,
        shared: bool,
    ) -> Self {
        let end = start + size as u64;
        let mut flags = VmaFlags::FILE_BACKED;

        if shared {
            flags = flags.union(VmaFlags::SHARED);
        }

        Self {
            start,
            end,
            protection: prot,
            flags,
            intent: MemoryIntent::FileMmap,
            file_offset: offset,
            file_id,
        }
    }

    /// Cria guard page
    pub fn new_guard(start: VirtAddr) -> Self {
        Self {
            start,
            end: start + 4096u64,
            protection: Protection::NONE,
            flags: VmaFlags::GUARD,
            intent: MemoryIntent::Guard,
            file_offset: 0,
            file_id: 0,
        }
    }

    /// Tamanho da VMA em bytes
    #[inline]
    pub fn size(&self) -> usize {
        (self.end.as_u64() - self.start.as_u64()) as usize
    }

    /// Número de páginas
    #[inline]
    pub fn page_count(&self) -> usize {
        self.size() / crate::rmm::config::PAGE_SIZE
    }

    /// Verifica se contém endereço
    #[inline]
    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr >= self.start && addr < self.end
    }

    /// Verifica se overlaps com outra VMA
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }

    /// Verifica se overlaps com range
    pub fn overlaps_range(&self, start: VirtAddr, end: VirtAddr) -> bool {
        self.start < end && self.end > start
    }

    /// Verifica se é adjacente (para merge)
    pub fn is_adjacent(&self, other: &Self) -> bool {
        self.end == other.start || other.end == self.start
    }

    /// Pode fazer merge com outra VMA?
    ///
    /// VMAs podem ser merged se são adjacentes e têm mesmas propriedades.
    pub fn can_merge(&self, other: &Self) -> bool {
        self.is_adjacent(other)
            && self.protection == other.protection
            && self.flags == other.flags
            && self.intent == other.intent
            && !self.flags.contains(VmaFlags::FILE_BACKED) // Não merge file-backed
    }

    /// Merge com VMA adjacente
    pub fn merge(&mut self, other: &Self) -> bool {
        if !self.can_merge(other) {
            return false;
        }

        if self.end == other.start {
            self.end = other.end;
        } else if other.end == self.start {
            self.start = other.start;
        } else {
            return false;
        }

        true
    }

    /// Split VMA em um ponto
    ///
    /// Retorna a parte após o split point.
    pub fn split(&mut self, at: VirtAddr) -> Option<Self> {
        if at <= self.start || at >= self.end {
            return None;
        }

        let new_vma = Self {
            start: at,
            end: self.end,
            protection: self.protection,
            flags: self.flags,
            intent: self.intent,
            file_offset: self.file_offset + (at.as_u64() - self.start.as_u64()),
            file_id: self.file_id,
        };

        self.end = at;

        Some(new_vma)
    }

    /// Verifica se pode crescer (para heap/stack)
    pub fn can_grow(&self) -> bool {
        self.flags.contains(VmaFlags::GROWABLE)
    }

    /// Cresce a VMA
    pub fn grow(&mut self, new_end: VirtAddr) -> bool {
        if !self.can_grow() {
            return false;
        }

        if self.flags.contains(VmaFlags::STACK) {
            // Stack cresce para baixo
            if new_end < self.start {
                self.start = new_end;
                return true;
            }
        } else {
            // Outros crescem para cima
            if new_end > self.end {
                self.end = new_end;
                return true;
            }
        }

        false
    }

    /// Converte proteção para MapFlags
    pub fn to_map_flags(&self) -> MapFlags {
        self.protection.to_map_flags()
    }

    /// É file-backed?
    #[inline]
    pub fn is_file_backed(&self) -> bool {
        self.flags.contains(VmaFlags::FILE_BACKED)
    }

    /// É shared?
    #[inline]
    pub fn is_shared(&self) -> bool {
        self.flags.contains(VmaFlags::SHARED)
    }

    /// É CoW?
    #[inline]
    pub fn is_cow(&self) -> bool {
        self.flags.contains(VmaFlags::COW)
    }
}

impl core::fmt::Debug for Vma {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "VMA {{ 0x{:012x}-0x{:012x} {:?} {:?} }}",
            self.start.as_u64(),
            self.end.as_u64(),
            self.protection,
            self.intent
        )
    }
}
