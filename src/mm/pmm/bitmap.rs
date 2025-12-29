#![allow(dead_code)]
//! Bitmap Frame Allocator
//!
//! Alocador de frames físicos usando um bitmap simples.
//! Ocupa 1 bit por frame de memória física.
//!
//! ## Características
//! * Simples e robusto (KISS)
//! * Baixo overhead de metadados (1 bit por 4KB)
//! * Suporta regiões de memória descontíguas (com mapeamento virtual contíguo)
//!
//! ## Implementação
//! O bitmap é armazenado em uma região de memória física reservada logo após o boot.
//!

use super::stats::PmmStats;
// use super::PhysFrame;
use crate::core::boot::handoff::{BootInfo, MemoryMapEntry, MemoryType};
use crate::mm::addr::{self, PhysAddr};
use crate::mm::pmm::FRAME_SIZE as PAGE_SIZE;
use core::sync::atomic::{compiler_fence, Ordering};

// ============================================================================
// CONFIGURAÇÃO
// ============================================================================

/// Número máximo de entradas no Memory Map para processar
const MAX_MEMORY_MAP_ENTRIES: usize = 128;

/// Padding de segurança para não sobrescrever o Kernel ou Bootloader (2MB)
const KERNEL_SAFETY_PADDING: u64 = 2 * 1024 * 1024;

/// Primeiro endereço físico que pode ser alocado (1MB)
/// O primeiro 1MB é historicamente problemático:
/// - 0x000-0x3FF: Real Mode IVT
/// - 0x400-0x4FF: BIOS Data Area (BDA)
/// - 0x500-0x7BFF: Área livre convencional (mas perigosa)
/// - 0x7C00-0x7DFF: MBR carregado pelo BIOS
/// - 0x80000-0x9FFFF: EBDA (Extended BIOS Data Area)
/// - 0xA0000-0xFFFFF: Video RAM, ROMs, etc.
const FIRST_ALLOCATABLE_ADDR: u64 = 1024 * 1024; // 1 MB

/// Primeiro frame que pode ser alocado (1MB / 4KB = 256)
const FIRST_ALLOCATABLE_FRAME: u64 = FIRST_ALLOCATABLE_ADDR / PAGE_SIZE;

/// Endereço mínimo para alocar o bitmap (evita região ISA DMA < 16MB se possível)
const MIN_REGION_ADDR: u64 = 16 * 1024 * 1024;

/// Limite de tentativas para probing de região livre
const MAX_PROBING_ATTEMPTS: usize = 64;

// ============================================================================
// ESTRUTURA
// ============================================================================

/// O Alocador de Bitmap
pub struct BitmapFrameAllocator {
    /// Ponteiro para o início do bitmap na memória VIRTUAL
    bitmap_ptr: *mut u64,
    /// Tamanho do bitmap em u64s (palavras de 64 bits)
    bitmap_len: usize,
    /// Total de frames gerenciados
    total_frames: usize,
    /// Estatísticas de uso
    stats: PmmStats,
    /// Lock simples (Spinlock seria ideal, mas PMM é muito baixo nível)
    /// Por enquanto, assumimos Single Core no boot ou lock externo.
    _lock: (),
}

// SAFETY: O alocador deve ser thread-safe (usar atomics ou lock externo)
unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}

impl BitmapFrameAllocator {
    /// Cria um novo alocador vazio (const para estáticos)
    pub const fn new() -> Self {
        Self {
            bitmap_ptr: core::ptr::null_mut(),
            bitmap_len: 0,
            total_frames: 0,
            stats: PmmStats::new(),
            _lock: (),
        }
    }

    /// Inicializa o alocador usando o BootInfo fornecido pelo Bootloader.
    ///
    /// # Etapas
    /// 1. Analisa o mapa de memória físico.
    /// 2. Calcula o tamanho necessário para o bitmap.
    /// 3. Encontra uma região de memória livre capaz de armazenar o bitmap.
    /// 4. Mapeia essa região e inicializa o bitmap (marca tudo como ocupado inicialmente).
    /// 5. Percorre o mapa de memória novamente e libera os frames marcados como `Usable`.
    pub fn init(&mut self, boot_info: &BootInfo) {
        crate::kinfo!("(PMM) Inicializando Bitmap Allocator...");

        // Barreira: Garante que a leitura do boot_info está completa
        compiler_fence(Ordering::SeqCst);

        // 1. Escanear Memory Map
        let (max_phys, _total_usable_frames) = self.scan_memory_map_safe(boot_info);

        // Barreira: Garante que max_phys está computado antes de usar
        compiler_fence(Ordering::SeqCst);

        // 2. Calcular tamanho do bitmap
        self.total_frames = (max_phys.as_u64() as usize) / (PAGE_SIZE as usize);
        self.stats.total_frames = self.total_frames;
        self.stats
            .used_frames
            .store(self.total_frames, Ordering::SeqCst);

        let bitmap_size_bytes = (self.total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;
        self.bitmap_len = bitmap_size_u64;
        let req_size_bytes = self.bitmap_len * 8;

        crate::ktrace!("(PMM) Total de quadros=", self.total_frames as u64);
        crate::ktrace!(
            "(PMM) Tamanho do bitmap KB=",
            (req_size_bytes / 1024) as u64
        );

        // Barreira: Garante que cálculos estão completos
        compiler_fence(Ordering::SeqCst);

        // 3. Encontrar região segura para o bitmap
        let bitmap_phys = self.find_bitmap_region_center_out(boot_info, req_size_bytes);
        crate::ktrace!("(PMM) Endereço físico do bitmap=", bitmap_phys.as_u64());

        // Barreira: Garante que bitmap_phys está definido
        compiler_fence(Ordering::SeqCst);

        // 4. Mapear e preencher bitmap
        // No estágio atual, assumimos identidade mapeada ou convertemos phys->virt linearmente
        // SAFETY: phys_to_virt é unsafe pois cria ponteiros arbitrários, mas aqui estamos mapeando região válida
        unsafe {
            let virt_addr = addr::phys_to_virt::<u64>(bitmap_phys.as_u64());
            self.bitmap_ptr = virt_addr;
        }

        if self.bitmap_ptr.is_null() || (self.bitmap_ptr as usize) % 8 != 0 {
            panic!("(PMM) Ponteiro de bitmap inválido: {:p}", self.bitmap_ptr);
        }

        // Barreira antes do memset crítico
        compiler_fence(Ordering::SeqCst);

        // Debug: imprimir valor do ponteiro
        crate::ktrace!("[PMM] bitmap_ptr=", self.bitmap_ptr as u64);
        crate::ktrace!("[PMM] bitmap_len=", self.bitmap_len as u64);

        // Memset: Preenche tudo como OCUPADO
        // Usando while manual em vez de for (iterador Range pode gerar #UD)
        let ptr_u64 = self.bitmap_ptr;

        let mut i: usize = 0;
        while i < self.bitmap_len {
            unsafe {
                let addr = ptr_u64.add(i);
                // Usar write_volatile ou ptr::write para evitar otimizações estranhas em early boot
                core::ptr::write(addr, u64::MAX);
            }
            i += 1;
        }

        // Barreira após memset
        compiler_fence(Ordering::SeqCst);

        // 4.5 CRÍTICO: Marcar page tables do bootloader como ocupadas
        // DEVE ser feito APÓS bitmap estar preenchido (tudo ocupado) e
        // ANTES de init_free_regions liberar frames "Usable".
        crate::ktrace!("(PMM) Escaneando tabelas de página do bootloader...");
        // SAFETY: mark_bootloader_page_tables é unsafe, assume self inicializado
        unsafe { super::pt_scanner::mark_bootloader_page_tables(self) };

        // 5. Liberar regiões usable
        self.init_free_regions(boot_info, bitmap_phys, req_size_bytes as u64);

        // Barreira final
        compiler_fence(Ordering::SeqCst);

        let used = self.stats.used_frames.load(Ordering::SeqCst);
        let free = self.total_frames.saturating_sub(used);

        crate::kinfo!(
            "(PMM) Inicialização completa. Total=",
            self.total_frames as u64
        );
        crate::kinfo!("(PMM) Livres=", free as u64);
    }

    /// Scan seguro do mapa de memória
    fn scan_memory_map_safe(&self, boot_info: &BootInfo) -> (PhysAddr, usize) {
        let mut max_phys = 0u64;
        let mut count = 0usize;

        let regions = unsafe {
            core::slice::from_raw_parts(
                boot_info.memory_map_addr as *const MemoryMapEntry,
                boot_info.memory_map_len as usize,
            )
        };

        for region in regions {
            if region.typ == MemoryType::Usable {
                let end = region.base + region.len;
                if end > max_phys {
                    max_phys = end;
                }
                count += 1;
            }
        }

        if max_phys == 0 {
            crate::kerror!("(PMM) FATAL: Nenhuma memória utilizável (Usable) encontrada!");
            return (PhysAddr::new(128 * 1024 * 1024), 0);
        }

        crate::kdebug!("(PMM) Memória física máxima=", max_phys);
        (PhysAddr::new(max_phys), count)
    }

    /// Estratégia "Center-Out" com Probing Fibonacci
    fn find_bitmap_region_center_out(&self, boot_info: &BootInfo, size_bytes: usize) -> PhysAddr {
        let size_needed = size_bytes as u64;
        // Padding para evitar problemas
        let forbidden_start = 0;
        let forbidden_end = KERNEL_SAFETY_PADDING; // Simplesmente evitar início

        let regions = unsafe {
            core::slice::from_raw_parts(
                boot_info.memory_map_addr as *const MemoryMapEntry,
                boot_info.memory_map_len as usize,
            )
        };

        // Encontrar a MAIOR região Usable
        let mut best_region_idx = None;
        let mut max_len = 0u64;

        for (i, region) in regions.iter().enumerate() {
            if region.typ == MemoryType::Usable
                && region.len > max_len
                && region.base >= MIN_REGION_ADDR
            {
                max_len = region.len;
                best_region_idx = Some(i);
            }
        }

        if let Some(idx) = best_region_idx {
            let region = &regions[idx];
            let region_start = region.base;
            let region_end = region.base + region.len;
            let region_center = region_start + (region.len / 2);
            let center_candidate = (region_center.saturating_sub(size_needed / 2) + 0xFFF) & !0xFFF;

            let mut fib_a = 0u64;
            let mut fib_b = PAGE_SIZE as u64;

            let mut attempt: usize = 0;
            while attempt < MAX_PROBING_ATTEMPTS {
                let offset = if attempt == 0 { 0 } else { fib_b };
                let sign = if attempt % 2 == 0 { 1i64 } else { -1i64 };

                if attempt > 0 && attempt % 2 == 0 {
                    let next = fib_a + fib_b;
                    fib_a = fib_b;
                    fib_b = next;
                }

                let candidate_start = (center_candidate as i64 + (offset as i64 * sign)) as u64;
                let candidate_end = candidate_start + size_needed;

                if candidate_start < region_start || candidate_end > region_end {
                    attempt += 1;
                    continue;
                }

                if candidate_start < forbidden_end && candidate_end > forbidden_start {
                    attempt += 1;
                    continue;
                }

                // Sucesso
                return PhysAddr::new(candidate_start);
            }
        }

        panic!("(PMM) Falha crítica: não foi possível alocar região para bitmap!");
    }

    /// Verifica se memória contém dados
    #[allow(dead_code)]
    unsafe fn is_memory_dirty(&self, start: u64, size: u64) -> bool {
        let ptr = addr::phys_to_virt::<u64>(start);
        let offsets = [0usize, (size / 2 / 8) as usize, ((size - 8) / 8) as usize];

        // Barreira antes de ler memória
        compiler_fence(Ordering::SeqCst);

        for &off in &offsets {
            let val = core::ptr::read_volatile(ptr.add(off));
            if val != 0 {
                return true;
            }
        }
        false
    }

    /// Inicializa regiões livres no bitmap
    fn init_free_regions(
        &mut self,
        boot_info: &BootInfo,
        bitmap_start_phys: PhysAddr,
        bitmap_size_bytes: u64,
    ) {
        let bitmap_end_phys = bitmap_start_phys.as_u64() + bitmap_size_bytes;

        let regions = unsafe {
            core::slice::from_raw_parts(
                boot_info.memory_map_addr as *const MemoryMapEntry,
                boot_info.memory_map_len as usize,
            )
        };

        for region in regions {
            // Só liberamos regiões marcadas como Usable
            if region.typ == MemoryType::Usable {
                let mut start = region.base;
                let mut end = region.base + region.len;

                // Alinhar ao tamanho da página
                start = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                end = end & !(PAGE_SIZE - 1);

                if start >= end {
                    continue;
                }

                // Excluir a região onde o próprio bitmap está!
                if start < bitmap_end_phys && end > bitmap_start_phys.as_u64() {
                    // Caso 1: Região engole o bitmap completamente
                    if start < bitmap_start_phys.as_u64() {
                        self.free_region(start, bitmap_start_phys.as_u64());
                    }
                    if end > bitmap_end_phys {
                        self.free_region(bitmap_end_phys, end);
                    }
                } else {
                    // Caso 2: Sem overlap, liberar tudo
                    self.free_region(start, end);
                }
            }
        }
    }

    /// Libera uma região contígua de memória física no bitmap
    fn free_region(&self, start: u64, end: u64) {
        let mut start_frame = start / PAGE_SIZE;
        let end_frame = end / PAGE_SIZE;

        // CRÍTICO: Nunca liberar frames no primeiro 1MB
        // Região historicamente problemática (IVT, BDA, EBDA, ROMs)
        if start_frame < FIRST_ALLOCATABLE_FRAME {
            start_frame = FIRST_ALLOCATABLE_FRAME;
        }

        // Se toda a região está abaixo de 1MB, não há nada a liberar
        if start_frame >= end_frame {
            return;
        }

        let mut frame = start_frame;
        while frame < end_frame {
            if frame >= self.total_frames as u64 {
                break;
            }

            // Marcar como livre (0)
            self.mark_frame(frame, false);
            self.stats.inc_free();
            frame += 1;
        }
    }

    /// Marca um frame como usado (true) ou livre (false)
    fn mark_frame(&self, frame_idx: u64, used: bool) {
        if frame_idx >= self.total_frames as u64 {
            return;
        }

        let word_idx = (frame_idx / 64) as usize;
        let bit_idx = (frame_idx % 64) as usize;
        let mask = 1u64 << bit_idx;

        // Operações atômicas manuais no bitmap (sem lock global por performance)
        // Isso é seguro porque cada bit é independente, mas words compartilhadas exigem cuidado.
        // Como estamos operando em words u64, load/store pode não ser suficiente para update parcial.
        // Deveríamos usar AtomicU64, mas o bitmap é um *mut u64 raw.
        // Vamos usar inline assembly lock bts/btr para x86_64 seria ideal,
        // ou um CAST para AtomicU64 se o alinhamento permitir.

        // Assumindo alinhamento correto do bitmap_ptr (verificado no init)
        unsafe {
            let word_ptr = self.bitmap_ptr.add(word_idx) as *mut core::sync::atomic::AtomicU64;
            let atomic = &*word_ptr;

            if used {
                atomic.fetch_or(mask, Ordering::Relaxed);
            } else {
                atomic.fetch_and(!mask, Ordering::Relaxed);
            }
        }
    }

    /// Aloca um frame físico
    pub fn allocate_frame(&self) -> Option<PhysAddr> {
        // Scan simples (Next Fit ou First Fit)
        // Para simplificar: First Fit com otimização de word

        // Começar busca a partir do frame 256 (1MB) para evitar região baixa
        let start_word = (FIRST_ALLOCATABLE_FRAME / 64) as usize;

        for i in start_word..self.bitmap_len {
            unsafe {
                let word_ptr = self.bitmap_ptr.add(i);
                let word = *word_ptr;

                // Se word for u64::MAX, está tudo ocupado
                if word == u64::MAX {
                    continue;
                }

                // Encontrar primeiro bit 0
                let _bit = word.trailing_ones(); // bits 0..N-1 são 1s, bit N é 0? Não.
                                                 // trailing_ones conta 1s consecutivos. Se word for 0, retorna 0. Se 0xFFFF...FE (bit 0=0), retorna 0.
                                                 // Se word tiver algum 0, trailing_ones de (!word) dá o índice do primeiro bit 1 em !word, ou seja, primeiro bit 0 em word?
                                                 // trailing_zeros(!word)

                let free_bit = (!word).trailing_zeros();
                if free_bit < 64 {
                    let frame_idx = (i as u64 * 64) + free_bit as u64;

                    if frame_idx >= self.total_frames as u64 {
                        return None;
                    }

                    // Tentar marcar atomicamente
                    let mask = 1u64 << free_bit;
                    let atomic_ptr = word_ptr as *mut core::sync::atomic::AtomicU64;
                    let atomic = &*atomic_ptr;

                    // CAS loop para garantir
                    let prev = atomic.fetch_or(mask, Ordering::AcqRel);
                    if (prev & mask) == 0 {
                        // Sucesso, estava livre e marcamos
                        self.stats.inc_alloc();
                        return Some(PhysAddr::new(frame_idx * PAGE_SIZE));
                    } else {
                        // Falha, alguém alocou antes. Tentar novamente ou continuar.
                        // Simplesmente continue a busca
                    }
                }
            }
        }

        self.stats.failed_allocs.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Desaloca um frame físico
    pub fn deallocate_frame(&self, frame: PhysAddr) {
        let frame_idx = frame.as_u64() / PAGE_SIZE;
        self.mark_frame(frame_idx, false);
        self.stats.inc_free();
    }

    /// Retorna o número total de frames físicos gerenciados
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Verifica se um frame específico está marcado como usado
    pub fn is_frame_used(&self, frame_idx: u64) -> bool {
        if frame_idx >= self.total_frames as u64 {
            return true; // Fora do intervalo é considerado usado/inválido
        }

        let word_idx = (frame_idx / 64) as usize;
        let bit_idx = (frame_idx % 64) as usize;
        let mask = 1u64 << bit_idx;

        unsafe {
            let word_ptr = self.bitmap_ptr.add(word_idx);
            let word = core::ptr::read_volatile(word_ptr);
            (word & mask) != 0
        }
    }

    /// Marca um frame específico como usado ou livre
    pub fn mark_frame_used(&mut self, frame_idx: u64, used: bool) {
        self.mark_frame(frame_idx, used);
    }
}
