//! # DOCUMENTAÇÃO CRÍTICA DE ARQUITETURA DE MEMÓRIA (PMM)
//!
//! ## 🛑 O Incidente do "Triple Fault" (Colisão de Memória)
//!
//! **Sintoma:** O sistema reiniciava abruptamente (Triple Fault) logo após o PMM iniciar e tentar
//! limpar o bitmap de memória (`memset`). Em alguns casos, ocorria `panic` por acesso a `null`.
//!
//! **A Causa Raiz:** O Bootloader aloca dinamicamente Tabelas de Página (Page Tables) em regiões
//! de memória marcadas como `Usable` no Memory Map, mas **não atualiza o mapa** para `Reserved` ou
//! `PageTable` antes de passar o controle para o Kernel. Essas tabelas frequentemente residem
//! logo após o fim do binário do Kernel ou nas bordas de regiões livres.
//!
//! Quando o PMM tentava alocar o Bitmap usando uma estratégia simples ("First Fit" ou "Append to Kernel"),
//! ele escolhia exatamente o mesmo endereço físico onde essas Page Tables ativas estavam vivendo.
//! Ao fazer o `memset` para limpar o bitmap, o PMM **sobrescrevia as tabelas de página que a CPU
//! estava usando para executar o próprio código**, puxando o tapete debaixo dos próprios pés.
//!
//! ## 🛡️ A Solução Robusta: "Center-Out" & "Fibonacci Probing"
//!
//! Para resolver isso sem depender da boa vontade do Bootloader, implementamos uma estratégia defensiva:
//!
//! 1.  **Ignorar Memória Baixa (< 16MB):** A região abaixo de 16MB (DMA/ISA) é historicamente instável e
//!     cheia de armadilhas de hardware legado. Nós a ignoramos completamente para estruturas críticas.
//! 2.  **Estratégia "Center-Out":** Em vez de pegar a primeira região livre (que geralmente é uma borda suja),
//!     calculamos o **centro geométrico** da maior região de RAM disponível. Estatisticamente, o centro
//!     de um grande bloco de 2GB+ é o lugar mais seguro e longe de qualquer alocação de borda do UEFI.
//! 3.  **Sonda Fibonacci (Probing):** Antes de aceitar um endereço, fazemos um `dirty check` (leitura volátil).
//!     Se a memória contiver dados não-nulos, assumimos que é "sujeira" do Bootloader e usamos offsets
//!     da sequência de Fibonacci para "espiralar" para fora daquele ponto até achar memória limpa (`0x00`).
//!
//! ## 🔮 TODO & Roadmap para Solução Definitiva
//!
//! Esta solução é resiliente, mas tecnicamente é um *workaround* inteligente. A solução canônica envolve:
//!
//! 1.  **Bootloader Protocol:** O Bootloader deve marcar explicitamente as regiões usadas por Page Tables
//!     como `LoaderPageTable` ou `Reserved` no Memory Map passado ao Kernel.
//! 2.  **Parse UEFI:** O Kernel poderia varrer a árvore de Page Tables ativa (via registro CR3) antes de
//!     iniciar o PMM e marcar manualmente esses quadros como ocupados no bitmap.
//! 3.  **Sanitização:** Implementar uma rotina de `sanitize_memory_map` que remove regiões muito pequenas
//!     ou funde regiões adjacentes antes do alocador rodar.
//!
//! ## ⚠️ O Que NÃO Fazer (Lições Aprendidas)
//!
//! * **NUNCA** confie cegamente que uma região `Usable` está realmente vazia, especialmente nas bordas.
//! * **NUNCA** aloque estruturas críticas do kernel em endereço físico `0x0` (causa panic no Rust).
//! * **NUNCA** tente alocar "logo após o kernel" sem garantir um padding generoso (2MB+), pois é onde
//!     o bootloader adora esconder coisas.
use super::frame::PhysFrame;
use super::stats::PmmStats;
use crate::core::handoff::{BootInfo, MemoryType};
use crate::mm::addr::{self, PhysAddr};
use crate::mm::config::PAGE_SIZE;
use core::sync::atomic::{compiler_fence, Ordering};

// ============================================================================
// CONSTANTES DE CONFIGURAÇÃO
// ============================================================================

/// Limite mínimo de endereço para alocação de estruturas críticas (16MB).
const MIN_ALLOC_ADDR: u64 = 16 * 1024 * 1024;

/// Limite mínimo para seleção de região (1MB).
const MIN_REGION_ADDR: u64 = 0x100000;

/// Padding de segurança ao redor do kernel (1MB).
const KERNEL_SAFETY_PADDING: u64 = 1024 * 1024;

/// Número máximo de entradas do Memory Map a processar.
const MAX_MEMORY_MAP_ENTRIES: usize = 128;

/// Número máximo de tentativas na estratégia Center-Out.
const MAX_PROBING_ATTEMPTS: usize = 20;

// ============================================================================
// ESTRUTURA PRINCIPAL
// ============================================================================

/// BitmapFrameAllocator - Gerencia memória física usando um bitmap.
pub struct BitmapFrameAllocator {
    _memory_base: PhysAddr,
    bitmap_ptr: *mut u64,
    bitmap_len: usize,
    total_frames: usize,
    next_free: usize,
    stats: PmmStats,
}

unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}

impl BitmapFrameAllocator {
    pub const fn empty() -> Self {
        Self {
            _memory_base: PhysAddr::new(0),
            bitmap_ptr: core::ptr::null_mut(),
            bitmap_len: 0,
            total_frames: 0,
            next_free: 0,
            stats: PmmStats::new(),
        }
    }

    /// Inicializa o alocador
    pub unsafe fn init(&mut self, boot_info: &'static BootInfo) {
        crate::kinfo!("(PMM) Inicializando BitmapFrameAllocator...");

        // Barreira: Garante que a leitura do boot_info está completa
        compiler_fence(Ordering::SeqCst);

        // 1. Escanear Memory Map
        let (max_phys, _) = self.scan_memory_map_safe(boot_info);

        // Barreira: Garante que max_phys está computado antes de usar
        compiler_fence(Ordering::SeqCst);

        // 2. Calcular tamanho do bitmap
        self.total_frames = max_phys.as_usize() / PAGE_SIZE;
        self.stats.total_frames = self.total_frames;
        self.stats
            .used_frames
            .store(self.total_frames, Ordering::SeqCst);

        let bitmap_size_bytes = (self.total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;
        self.bitmap_len = bitmap_size_u64;
        let req_size_bytes = self.bitmap_len * 8;

        crate::kdebug!(
            "(PMM) total_frames={}, bitmap_size={}KB",
            self.total_frames,
            req_size_bytes / 1024
        );

        // Barreira: Garante que cálculos estão completos
        compiler_fence(Ordering::SeqCst);

        // 3. Encontrar região segura para o bitmap
        let bitmap_phys = self.find_bitmap_region_center_out(boot_info, req_size_bytes);
        crate::kdebug!("(PMM) bitmap_phys={:#x}", bitmap_phys.as_u64());

        // Barreira: Garante que bitmap_phys está definido
        compiler_fence(Ordering::SeqCst);

        // 4. Mapear e preencher bitmap
        self.bitmap_ptr = addr::phys_to_virt(bitmap_phys).as_mut_ptr();

        if self.bitmap_ptr.is_null() || (self.bitmap_ptr as usize) % 8 != 0 {
            panic!("(PMM) Ponteiro de bitmap inválido: {:p}", self.bitmap_ptr);
        }

        // Barreira antes do memset crítico
        compiler_fence(Ordering::SeqCst);

        // Memset: Preenche tudo como OCUPADO
        let ptr_u64 = self.bitmap_ptr;
        for i in 0..self.bitmap_len {
            core::ptr::write_volatile(ptr_u64.add(i), u64::MAX);
        }

        // Barreira após memset
        compiler_fence(Ordering::SeqCst);

        // 4.5 CRÍTICO: Marcar page tables do bootloader como ocupadas
        // DEVE ser feito APÓS bitmap estar preenchido (tudo ocupado) e
        // ANTES de init_free_regions liberar frames "Usable".
        // Isso garante que as page tables do bootloader nunca sejam
        // marcadas como livres, mesmo que estejam em regiões "Usable".
        crate::kdebug!("(PMM) Escaneando page tables do bootloader...");
        super::pt_scanner::mark_bootloader_page_tables(self);

        // 5. Liberar regiões usable
        self.init_free_regions(boot_info, bitmap_phys, req_size_bytes as u64);

        // Barreira final
        compiler_fence(Ordering::SeqCst);

        let used = self.stats.used_frames.load(Ordering::SeqCst);
        let free = self.total_frames.saturating_sub(used);

        crate::kinfo!(
            "(PMM) Init completo. Total: {} frames. Livres: {}",
            self.total_frames,
            free
        );
    }

    /// Scan seguro do mapa de memória
    fn scan_memory_map_safe(&self, boot_info: &BootInfo) -> (PhysAddr, usize) {
        let mut max_phys = 0u64;
        let mut count = 0usize;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;
        let safe_len = core::cmp::min(map_len, MAX_MEMORY_MAP_ENTRIES);

        // Barreira antes de iterar sobre memória externa
        compiler_fence(Ordering::SeqCst);

        for i in 0..safe_len {
            unsafe {
                let entry = &*map_ptr.add(i);
                if entry.typ == MemoryType::Usable {
                    let end = entry.base + entry.len;
                    if end > max_phys {
                        max_phys = end;
                    }
                    count += 1;
                }
            }
        }

        // Barreira após leitura
        compiler_fence(Ordering::SeqCst);

        if max_phys == 0 {
            crate::kerror!("(PMM) FATAL: Nenhuma memória Usable encontrada!");
            return (PhysAddr::new(128 * 1024 * 1024), 0);
        }

        crate::kdebug!("(PMM) max_phys={:#x}", max_phys);
        (PhysAddr::new(max_phys), count)
    }

    /// Estratégia "Center-Out" com Probing Fibonacci
    fn find_bitmap_region_center_out(&self, boot_info: &BootInfo, size_bytes: usize) -> PhysAddr {
        let kernel_start = boot_info.kernel_phys_addr;
        let kernel_end = boot_info.kernel_size + kernel_start;
        let size_needed = size_bytes as u64;

        let forbidden_start = kernel_start.saturating_sub(KERNEL_SAFETY_PADDING);
        let forbidden_end = kernel_end + KERNEL_SAFETY_PADDING;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = core::cmp::min(boot_info.memory_map_len as usize, MAX_MEMORY_MAP_ENTRIES);

        // Barreira
        compiler_fence(Ordering::SeqCst);

        // Encontrar a MAIOR região Usable
        let mut best_region_idx = None;
        let mut max_len = 0u64;

        for i in 0..map_len {
            unsafe {
                let entry = &*map_ptr.add(i);
                if entry.typ == MemoryType::Usable
                    && entry.len > max_len
                    && entry.base >= MIN_REGION_ADDR
                {
                    max_len = entry.len;
                    best_region_idx = Some(i);
                }
            }
        }

        // Barreira
        compiler_fence(Ordering::SeqCst);

        if let Some(idx) = best_region_idx {
            let entry = unsafe { &*map_ptr.add(idx) };
            let region_start = entry.base;
            let region_end = entry.base + entry.len;
            let region_center = region_start + (entry.len / 2);
            let center_candidate = (region_center.saturating_sub(size_needed / 2) + 0xFFF) & !0xFFF;

            let mut fib_a = 0u64;
            let mut fib_b = PAGE_SIZE as u64;

            for attempt in 0..MAX_PROBING_ATTEMPTS {
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
                    continue;
                }

                if candidate_start < forbidden_end && candidate_end > forbidden_start {
                    continue;
                }

                // Barreira antes da verificação de memória
                compiler_fence(Ordering::SeqCst);

                if unsafe { self.is_memory_dirty(candidate_start, size_needed) } {
                    continue;
                }

                return PhysAddr::new(candidate_start);
            }
        }

        panic!("(PMM) Falha crítica: não foi possível alocar região para bitmap!");
    }

    /// Verifica se memória contém dados
    unsafe fn is_memory_dirty(&self, start: u64, size: u64) -> bool {
        let ptr = addr::phys_to_virt(PhysAddr::new(start)).as_mut_ptr() as *const u64;
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

    /// Aloca um frame físico
    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let start_search = self.next_free;
        let limit = self.bitmap_len;

        for i in 0..limit {
            let idx = (start_search + i) % limit;
            unsafe {
                let entry_ptr = self.bitmap_ptr.add(idx);
                let entry = core::ptr::read_volatile(entry_ptr);

                if entry != u64::MAX {
                    let bit = entry.trailing_ones() as usize;
                    if bit < 64 {
                        let frame_idx = idx * 64 + bit;
                        if frame_idx < self.total_frames {
                            core::ptr::write_volatile(entry_ptr, entry | (1 << bit));
                            compiler_fence(Ordering::SeqCst);
                            self.stats.inc_alloc();
                            self.next_free = idx;
                            return Some(PhysFrame::from_start_address(PhysAddr::new(
                                frame_idx as u64 * PAGE_SIZE as u64,
                            )));
                        }
                    }
                }
            }
        }
        None
    }

    /// Desaloca um frame físico
    pub fn deallocate_frame(&mut self, frame: PhysFrame) {
        let start_addr = frame.start_address().as_u64();
        let frame_idx = (start_addr / PAGE_SIZE as u64) as usize;
        if frame_idx >= self.total_frames {
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            let entry = core::ptr::read_volatile(ptr);
            let mask = 1u64 << bit;

            if (entry & mask) != 0 {
                core::ptr::write_volatile(ptr, entry & !mask);
                compiler_fence(Ordering::SeqCst);
                self.stats.inc_free();
                if idx < self.next_free {
                    self.next_free = idx;
                }
            }
        }
    }

    /// Libera frames em regiões Usable
    unsafe fn init_free_regions(
        &mut self,
        boot_info: &BootInfo,
        bitmap_start: PhysAddr,
        bitmap_size: u64,
    ) {
        let kernel_start = boot_info.kernel_phys_addr;
        let kernel_end = kernel_start + boot_info.kernel_size;
        let bitmap_end = bitmap_start.as_u64() + bitmap_size;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = core::cmp::min(boot_info.memory_map_len as usize, MAX_MEMORY_MAP_ENTRIES);

        compiler_fence(Ordering::SeqCst);

        for i in 0..map_len {
            let entry = &*map_ptr.add(i);
            if entry.typ == MemoryType::Usable {
                let start_frame = entry.base / PAGE_SIZE as u64;
                let end_frame = (entry.base + entry.len) / PAGE_SIZE as u64;

                for f in start_frame..end_frame {
                    let addr = f * PAGE_SIZE as u64;

                    if addr < MIN_ALLOC_ADDR {
                        continue;
                    }
                    if addr >= kernel_start && addr < kernel_end {
                        continue;
                    }
                    if addr >= bitmap_start.as_u64() && addr < bitmap_end {
                        continue;
                    }

                    self.deallocate_frame_internal(f as usize);
                }
            }
        }

        compiler_fence(Ordering::SeqCst);
    }

    /// Desaloca frame internamente
    fn deallocate_frame_internal(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            return;
        }
        let idx = frame_idx / 64;
        let bit = frame_idx % 64;
        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            let entry = core::ptr::read_volatile(ptr);
            core::ptr::write_volatile(ptr, entry & !(1u64 << bit));
        }
        self.stats.used_frames.fetch_sub(1, Ordering::SeqCst);
    }

    // ========================================================================
    // MÉTODOS PARA PROTEÇÃO DE PAGE TABLES DO BOOTLOADER
    // ========================================================================

    /// Marca um frame específico como ocupado (usado pelo pt_scanner).
    ///
    /// Usado para proteger page tables do bootloader que estão em regiões
    /// marcadas como "Usable" no memory map.
    ///
    /// # Retorna
    /// - true se o frame foi marcado com sucesso
    /// - false se o frame já estava ocupado ou índice inválido
    pub fn mark_frame_used(&mut self, phys_addr: u64) -> bool {
        let frame_idx = (phys_addr / PAGE_SIZE as u64) as usize;

        if frame_idx >= self.total_frames {
            return false;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            let entry = core::ptr::read_volatile(ptr);

            // Verificar se já está ocupado
            if (entry & (1u64 << bit)) != 0 {
                return false; // Já estava marcado
            }

            // Marcar como ocupado
            core::ptr::write_volatile(ptr, entry | (1u64 << bit));
        }

        self.stats.used_frames.fetch_add(1, Ordering::SeqCst);
        true
    }

    /// Verifica se um frame está ocupado
    pub fn is_frame_used(&self, phys_addr: u64) -> bool {
        let frame_idx = (phys_addr / PAGE_SIZE as u64) as usize;

        if frame_idx >= self.total_frames {
            return true; // Fora do range = considerado ocupado
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            let entry = core::ptr::read_volatile(ptr);
            (entry & (1u64 << bit)) != 0
        }
    }
}
