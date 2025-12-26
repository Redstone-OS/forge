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
//!
//! ## ⚠️ NOTA SOBRE LOGS [INSANITY]
//!
//! Os logs marcados com `[INSANITY]` são CRÍTICOS e NÃO DEVEM SER REMOVIDOS.
//! Eles atuam como barreiras de sincronização que previnem otimizações agressivas
//! do compilador que causam o sistema travar. Isso indica um possível problema
//! de timing ou UB (Undefined Behavior) que precisa ser investigado futuramente.

use super::frame::PhysFrame;
use super::stats::PmmStats;
use crate::core::handoff::{BootInfo, MemoryType};
use crate::mm::addr::{self, PhysAddr};
use crate::mm::config::PAGE_SIZE;
use core::sync::atomic::Ordering;

// ============================================================================
// CONSTANTES DE CONFIGURAÇÃO
// ============================================================================

/// Limite mínimo de endereço para alocação de estruturas críticas (16MB).
/// Evita a região DMA/ISA legada que é historicamente problemática.
const MIN_ALLOC_ADDR: u64 = 16 * 1024 * 1024;

/// Limite mínimo para seleção de região (1MB - evita região legada no scan).
const MIN_REGION_ADDR: u64 = 0x100000;

/// Padding de segurança ao redor do kernel (1MB).
const KERNEL_SAFETY_PADDING: u64 = 1024 * 1024;

/// Número máximo de entradas do Memory Map a processar.
const MAX_MEMORY_MAP_ENTRIES: usize = 128;

/// Número máximo de tentativas na estratégia Center-Out / Fibonacci Probing.
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
        crate::ktrace!("(PMM) [INSANITY] >>> ENTER init()");
        crate::kinfo!(
            "(PMM) Inicializando BitmapFrameAllocator (Center-Out + Probe + StatsFix)..."
        );

        // 1. Calcular memória total com segurança
        crate::kdebug!("(PMM) [INSANITY] Passo 1: Escaneando memory map...");
        let (max_phys, _) = self.scan_memory_map_safe(boot_info);
        crate::kdebug!(
            "(PMM) [INSANITY] max_phys calculado: {:#x}",
            max_phys.as_u64()
        );

        // 2. Calcular tamanho do bitmap e inicializar estatísticas
        self.total_frames = max_phys.as_usize() / PAGE_SIZE;
        self.stats.total_frames = self.total_frames;

        // CRÍTICO: Inicializar used_frames = total porque bitmap começa com tudo ocupado
        self.stats
            .used_frames
            .store(self.total_frames, Ordering::Relaxed);

        let bitmap_size_bytes = (self.total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;
        self.bitmap_len = bitmap_size_u64;
        let req_size_bytes = self.bitmap_len * 8;

        crate::kdebug!(
            "(PMM) total_frames={}, bitmap_size={}KB, init_used={}",
            self.total_frames,
            req_size_bytes / 1024,
            self.total_frames
        );

        // 3. Alocar região física usando Center-Out com Probing
        crate::kdebug!("(PMM) [INSANITY] Passo 3: Buscando região segura (Probe Ativo)...");
        let bitmap_phys = self.find_bitmap_region_center_out(boot_info, req_size_bytes);
        crate::kdebug!(
            "(PMM) [INSANITY] Região ELEITA: bitmap_phys={:#x}",
            bitmap_phys.as_u64()
        );

        // 4. Mapear (HHDM) e limpar bitmap
        crate::kdebug!("(PMM) [INSANITY] Passo 4: Mapeando e limpando...");
        self.bitmap_ptr = addr::phys_to_virt(bitmap_phys).as_mut_ptr();

        if self.bitmap_ptr.is_null() || (self.bitmap_ptr as usize) % 8 != 0 {
            crate::kerror!(
                "(PMM) [INSANITY] FATAL: Ponteiro de bitmap inválido: {:p}",
                self.bitmap_ptr
            );
            panic!("PMM Bitmap Pointer Error");
        }

        crate::ktrace!("(PMM) [INSANITY] Bitmap Virt Addr: {:p}", self.bitmap_ptr);

        // Memset Seguro (u64) - Preenche tudo como OCUPADO (1)
        let ptr_u64 = self.bitmap_ptr;
        for i in 0..self.bitmap_len {
            core::ptr::write_volatile(ptr_u64.add(i), u64::MAX);
            if i > 0 && i % 4096 == 0 {
                crate::ktrace!(
                    "(PMM) [INSANITY] Memset progress: {}/{}",
                    i,
                    self.bitmap_len
                );
            }
        }
        crate::kdebug!("(PMM) [INSANITY] Memset completo.");

        // 5. Liberar regiões usable
        crate::kdebug!("(PMM) [INSANITY] Passo 6: Liberando frames disponíveis...");
        self.init_free_regions(boot_info, bitmap_phys, req_size_bytes as u64);

        let used = self.stats.used_frames.load(Ordering::Relaxed);
        let free = if self.total_frames >= used {
            self.total_frames - used
        } else {
            crate::kwarn!(
                "(PMM) [INSANITY] WARN: Estatística inconsistente: Total {} < Used {}",
                self.total_frames,
                used
            );
            0
        };

        crate::kinfo!(
            "(PMM) [INSANITY] Init completo. Total: {} frames. Usados: {}. Livres: {}",
            self.total_frames,
            used,
            free
        );
    }

    /// Scan seguro do mapa de memória
    fn scan_memory_map_safe(&self, boot_info: &BootInfo) -> (PhysAddr, usize) {
        crate::ktrace!("(PMM) [INSANITY] >>> ENTER scan_memory_map_safe");
        let mut max_phys = 0;
        let mut count = 0;
        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;

        crate::ktrace!(
            "(PMM) [INSANITY] Map Info: Ptr={:p} Len={}",
            map_ptr,
            map_len
        );

        if map_len > 512 {
            crate::kwarn!(
                "(PMM) [INSANITY] Map Len suspeito ({})! Limitando scan a {} entradas.",
                map_len,
                MAX_MEMORY_MAP_ENTRIES
            );
        }
        let safe_len = core::cmp::min(map_len, MAX_MEMORY_MAP_ENTRIES);

        for i in 0..safe_len {
            unsafe {
                let entry = &*map_ptr.add(i);

                if i < 8 || entry.typ == MemoryType::Usable {
                    crate::ktrace!(
                        "(PMM) [INSANITY] Entry[{}]: Base={:#x} Len={:#x} Type={:?}",
                        i,
                        entry.base,
                        entry.len,
                        entry.typ
                    );
                }

                if entry.typ == MemoryType::Usable {
                    let end = entry.base + entry.len;
                    if end > max_phys {
                        max_phys = end;
                    }
                    count += 1;
                }
            }
        }

        if max_phys == 0 {
            crate::kerror!("(PMM) [INSANITY] FATAL: Nenhuma memória Usable encontrada!");
            return (PhysAddr::new(128 * 1024 * 1024), 0);
        }

        (PhysAddr::new(max_phys), count)
    }

    /// Estratégia "Center-Out" com Probing Fibonacci
    fn find_bitmap_region_center_out(&self, boot_info: &BootInfo, size_bytes: usize) -> PhysAddr {
        let kernel_start = boot_info.kernel_phys_addr;
        let kernel_end = boot_info.kernel_size + kernel_start;
        let size_needed = size_bytes as u64;

        // Zona proibida expandida (Kernel ± padding)
        let forbidden_start = kernel_start.saturating_sub(KERNEL_SAFETY_PADDING);
        let forbidden_end = kernel_end + KERNEL_SAFETY_PADDING;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = core::cmp::min(boot_info.memory_map_len as usize, MAX_MEMORY_MAP_ENTRIES);

        // Encontrar a MAIOR região Usable acima de 1MB
        let mut best_region_idx = None;
        let mut max_len = 0;

        for i in 0..map_len {
            unsafe {
                let entry = &*map_ptr.add(i);
                if entry.typ == MemoryType::Usable {
                    if entry.len > max_len && entry.base >= MIN_REGION_ADDR {
                        max_len = entry.len;
                        best_region_idx = Some(i);
                    }
                }
            }
        }

        if let Some(idx) = best_region_idx {
            let entry = unsafe { &*map_ptr.add(idx) };
            let region_start = entry.base;
            let region_end = entry.base + entry.len;
            let region_center = region_start + (entry.len / 2);

            // Início ideal: Centro da região, alinhado a 4KB
            let center_candidate = (region_center.saturating_sub(size_needed / 2) + 0xFFF) & !0xFFF;

            crate::ktrace!(
                "(PMM) [INSANITY] Região Alvo: {:#x}-{:#x}. Centro: {:#x}",
                region_start,
                region_end,
                center_candidate
            );

            // Sequência Fibonacci para probing
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
                    crate::ktrace!(
                        "(PMM) [INSANITY] Attempt {} ({:#x}) fora da região. Skip.",
                        attempt,
                        candidate_start
                    );
                    continue;
                }

                if candidate_start < forbidden_end && candidate_end > forbidden_start {
                    crate::ktrace!(
                        "(PMM) [INSANITY] Attempt {} ({:#x}) colide com Kernel. Skip.",
                        attempt,
                        candidate_start
                    );
                    continue;
                }

                if unsafe { self.is_memory_dirty(candidate_start, size_needed) } {
                    crate::kwarn!(
                        "(PMM) [INSANITY] Attempt {} ({:#x}) é memória SUJA! Tentando próximo...",
                        attempt,
                        candidate_start
                    );
                    continue;
                }

                crate::kdebug!(
                    "(PMM) [INSANITY] VENCEDOR (Attempt {}): {:#x}",
                    attempt,
                    candidate_start
                );
                return PhysAddr::new(candidate_start);
            }
        }

        crate::kerror!("(PMM) [INSANITY] FATAL: Falha crítica na alocação Center-Out!");
        panic!("PMM OOM");
    }

    /// Verifica se memória contém dados (está "suja")
    unsafe fn is_memory_dirty(&self, start: u64, size: u64) -> bool {
        let ptr = addr::phys_to_virt(PhysAddr::new(start)).as_mut_ptr() as *const u64;
        let offsets = [0, size / 2 / 8, (size - 8) / 8];

        for &off in &offsets {
            let val = core::ptr::read_volatile(ptr.add(off as usize));
            if val != 0 {
                return true;
            }
        }
        false
    }

    /// Aloca um frame físico
    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let start_search = self.next_free;
        let mut i = 0;
        let limit = self.bitmap_len;

        while i < limit {
            let idx = (start_search + i) % limit;
            unsafe {
                let entry_ptr = self.bitmap_ptr.add(idx);
                let entry = *entry_ptr;
                if entry != u64::MAX {
                    let bit = entry.trailing_ones() as usize;
                    if bit < 64 {
                        let frame_idx = idx * 64 + bit;
                        if frame_idx < self.total_frames {
                            *entry_ptr |= 1 << bit;
                            self.stats.inc_alloc();
                            self.next_free = idx;
                            return Some(PhysFrame::from_start_address(PhysAddr::new(
                                frame_idx as u64 * PAGE_SIZE as u64,
                            )));
                        }
                    }
                }
            }
            i += 1;
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
            let mask = 1 << bit;
            if (*ptr & mask) != 0 {
                *ptr &= !mask;
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

        for i in 0..map_len {
            let entry = &*map_ptr.add(i);
            if entry.typ == MemoryType::Usable {
                let start_frame = entry.base / PAGE_SIZE as u64;
                let end_frame = (entry.base + entry.len) / PAGE_SIZE as u64;

                for f in start_frame..end_frame {
                    let addr = f * PAGE_SIZE as u64;

                    // Skip regiões protegidas
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
    }

    /// Desaloca frame internamente (usado na inicialização)
    fn deallocate_frame_internal(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            return;
        }
        let idx = frame_idx / 64;
        let bit = frame_idx % 64;
        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            *ptr &= !(1 << bit);
        }
        self.stats.used_frames.fetch_sub(1, Ordering::Relaxed);
    }
}
