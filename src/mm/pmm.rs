//! # Physical Memory Manager (PMM)
//!
//! O `PMM` é responsável por rastrear a posse de todos os frames de memória física (4 KiB) do sistema.
//! Ele serve como a "fonte da verdade" sobre quais blocos de RAM estão livres ou ocupados.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Alocação de Frames:** Oferecer frames livres para o VMM (para criar Page Tables ou mapear memória).
//! - **Contabilidade:** Rastrear uso de memória e detectar OOM (Out-Of-Memory).
//! - **Bootstrapping:** Inicializar-se a partir do Memory Map cru fornecido pelo bootloader.
//!
//! ## 🏗️ Arquitetura Interna (Bitmap Allocator)
//! O PMM utiliza um **Bitmap Global** linear:
//! - Cada bit corresponde a um frame de 4 KiB.
//! - `0` = Livre, `1` = Ocupado.
//! - O bitmap reside na própria memória física (alocado dinamicamente durante `init`).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Eficiência Espacial:** O overhead é mínimo (1 bit por 4 KiB = ~0.003% da RAM).
//!   - Ex: 4GB RAM requer apenas 128 KB de bitmap.
//! - **Simplicidade de Init:** Não requer estruturas complexas como árvores ou listas ligadas antes do heap existir.
//! - **Robustez:** Protege a si mesmo e ao kernel durante a inicialização, marcando suas próprias regiões como ocupadas.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Linear Scan (Performance):** A alocação (`allocate_frame`) faz uma busca linear no array de `u64`.
//!   - *Pior caso:* O(N/64). Em 32GB RAM, isso pode ser lento se a memória estiver fragmentada.
//! - **Single Global Lock:** O `Mutex<BitmapFrameAllocator>` é um ponto de contenção em multicore.
//! - **Fragmentação Física:** Não há suporte para alocar "blocos contíguos" (ex: para DMA drivers que precisam de buffers fisicamente contíguos > 4KiB).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Optimization)** Implementar "Search Pointer (Next Fit)" mais inteligente ou Hierarchical Bitmap.
//!   - *Meta:* Reduzir tempo de busca de O(N) para O(1) amortizado.
//! - [ ] **TODO: (Features)** Implementar `allocate_contiguous_frames(n)`.
//!   - *Motivo:* Drivers de vídeo/rede frequentemente precisam de buffers DMA contíguos.
//! - [ ] **TODO: (Arch)** Suporte a **NUMA (Non-Uniform Memory Access)**.
//!   - *Futuro:* Ter um PMM por nó NUMA para reduzir latência de acesso à memória.
//! - [ ] **TODO: (Reliability)** Detectar e isolar "Bad RAM" informada pelo firmware/bootloader.

use crate::core::handoff::BootInfo;
use crate::sync::Mutex;

pub const FRAME_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    /// Endereço físico do frame (alinhado a FRAME_SIZE).
    pub addr: u64,
}

impl PhysFrame {
    /// Retorna o frame que contém o endereço físico fornecido (alinhamento por floor).
    pub fn containing_address(addr: u64) -> Self {
        Self {
            addr: addr - (addr % FRAME_SIZE as u64),
        }
    }
}

/// Alocador global (embalado por Mutex para uso seguro entre contextos).
pub static FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator> = Mutex::new(BitmapFrameAllocator::empty());

/// BitmapFrameAllocator
///
/// Layout interno:
/// - `bitmap` é uma fatia de u64; cada bit (LSB = bit 0) representa um frame.
/// - `total_frames` é o número total de frames representados no bitmap.
/// - `memory_base` é o endereço físico base considerado para frame index 0.
///   (por simplicidade aqui usamos base = 0; poderia ser ajustado para offsets)
pub struct BitmapFrameAllocator {
    /// Início da região de memória gerenciada pelo bitmap.
    memory_base: u64,
    /// Bitmap onde cada bit representa um frame de 4KiB.
    /// 0 = Livre, 1 = Usado.
    bitmap: &'static mut [u64],
    /// Total de frames gerenciados.
    total_frames: usize,
    /// Dica para a próxima alocação (round-robin simples).
    next_free: usize,
    /// Estatística de uso.
    used_frames: usize,
}

impl BitmapFrameAllocator {
    /// Cria um alocador vazio — usado para inicialização estática.
    const fn empty() -> Self {
        Self {
            memory_base: 0,
            bitmap: &mut [],
            total_frames: 0,
            next_free: 0,
            used_frames: 0,
        }
    }

    // -------------------------
    // Inicialização
    // -------------------------

    /// Inicializa o alocador baseado no `BootInfo`.
    ///
    /// # Safety
    /// - `boot_info` deve apontar para um memory_map válido com `memory_map_len` entradas.
    /// - A função colocará o bitmap físico numa região Usable grande o suficiente.
    /// - Assume que não há race com outras inicializações do PMM.
    pub unsafe fn init(&mut self, boot_info: &'static BootInfo) {
        crate::kdebug!("(PMM) init: Iniciando...");
        crate::ktrace!("(PMM) init: boot_info={:p}", boot_info);
        crate::ktrace!("(PMM) init: memory_map_len={}", boot_info.memory_map_len);

        if boot_info.memory_map_len == 0 {
            crate::kerror!("(PMM) init: ERRO FATAL - mapa de memória vazio!");
            panic!("BootInfo não contém mapa de memória válido!");
        }

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;
        crate::ktrace!("(PMM) init: map_ptr={:p}, map_len={}", map_ptr, map_len);

        let regions = core::slice::from_raw_parts(map_ptr, map_len);

        // 1. Calcular memória total (apenas RAM utilizável, ignora MMIO)
        let mut max_phys_addr = 0;
        for region in regions {
            if region.typ == crate::core::handoff::MemoryType::Usable {
                let end = region.base + region.len;
                if end > max_phys_addr {
                    max_phys_addr = end;
                }
            }
        }

        crate::kinfo!(
            "(PMM) Memória máxima: {:#x} ({} MB)",
            max_phys_addr,
            max_phys_addr / 1024 / 1024
        );

        // 2) Calcular espaço do bitmap (em bytes -> em u64 entries)
        let total_frames = (max_phys_addr as usize) / FRAME_SIZE;
        let bitmap_size_bytes = (total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;
        let bitmap_total_size = bitmap_size_u64 * 8;

        let mut bitmap_phys_addr: u64 = 0;
        let mut best_region_size: u64 = 0;

        for region in regions.iter() {
            if region.typ == crate::core::handoff::MemoryType::Usable {
                if region.len >= bitmap_total_size as u64 && region.len > best_region_size {
                    let region_end = region.base + region.len;
                    let aligned_start =
                        (region_end - bitmap_total_size as u64) & !(FRAME_SIZE as u64 - 1);

                    if aligned_start >= region.base {
                        bitmap_phys_addr = aligned_start;
                        best_region_size = region.len;
                    }
                }
            }
        }

        if bitmap_phys_addr == 0 {
            panic!("(PMM) Não foi possível encontrar região Usable para o bitmap!");
        }

        crate::kinfo!(
            "(PMM) Bitmap em {:#x}, {} frames",
            bitmap_phys_addr,
            total_frames
        );

        // CRÍTICO: usar phys_to_virt para acessar o bitmap via identity map
        let bitmap_ptr = crate::mm::addr::phys_to_virt::<u64>(bitmap_phys_addr);
        self.bitmap = core::slice::from_raw_parts_mut(bitmap_ptr, bitmap_size_u64);
        self.bitmap.fill(u64::MAX); // Marcar tudo como ocupado

        self.memory_base = 0;
        self.total_frames = total_frames;
        self.used_frames = total_frames;

        // 3. Liberar regiões usáveis (protegendo kernel, bitmap e primeiros 16MB)
        let kernel_end = boot_info.kernel_phys_addr + boot_info.kernel_size;
        let bitmap_end = bitmap_phys_addr + (bitmap_size_u64 * 8) as u64;
        const MIN_USABLE_ADDR: u64 = 0x1000000; // 16 MB

        for region in regions {
            if region.typ == crate::core::handoff::MemoryType::Usable {
                let start_frame = region.base / FRAME_SIZE as u64;
                let end_frame = (region.base + region.len) / FRAME_SIZE as u64;

                for frame_idx in start_frame..end_frame {
                    let addr = frame_idx * FRAME_SIZE as u64;

                    if addr == 0 {
                        continue;
                    }
                    if addr < MIN_USABLE_ADDR {
                        continue;
                    }
                    if addr >= boot_info.kernel_phys_addr && addr < kernel_end {
                        continue;
                    }
                    if addr >= bitmap_phys_addr && addr < bitmap_end {
                        continue;
                    }

                    if frame_idx < total_frames as u64 {
                        self.deallocate_frame(frame_idx as usize);
                    }
                }
            }
        }

        crate::kinfo!(
            "(PMM) Inicializado: {} frames totais, {} livres",
            self.total_frames,
            self.total_frames - self.used_frames
        );
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Busca linear simples com "next_free" optimization
        let start_search = self.next_free;

        for i in 0..self.bitmap.len() {
            let idx = (start_search + i) % self.bitmap.len();
            let entry = self.bitmap[idx];

            if entry != u64::MAX {
                // Se não está tudo cheio (todos 1s)
                // Encontrar o bit 0 (livre)
                let bit = entry.trailing_ones() as usize;

                let frame_idx = idx * 64 + bit;

                if frame_idx < self.total_frames {
                    // Marcar como usado
                    self.bitmap[idx] |= 1 << bit;
                    self.used_frames += 1;
                    self.next_free = idx;

                    let addr = frame_idx as u64 * FRAME_SIZE as u64;
                    return Some(PhysFrame { addr });
                }
            }
        }

        // OOM (Out of Memory) - SEMPRE logar erros
        crate::kerror!(
            "(PMM) OOM! used={}/{} ({}% utilizado)",
            self.used_frames,
            self.total_frames,
            (self.used_frames * 100) / self.total_frames
        );

        None
    }

    pub fn deallocate_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            crate::kwarn!(
                "(PMM) deallocate: frame {} fora do range (max {})",
                frame_idx,
                self.total_frames
            );
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        // Verificar se já estava livre (Double Free)
        if (self.bitmap[idx] & (1 << bit)) == 0 {
            crate::kwarn!(
                "(PMM) DOUBLE FREE! frame={} addr={:#x}",
                frame_idx,
                frame_idx as u64 * FRAME_SIZE as u64
            );
            return;
        }

        // Marcar como livre (0)
        self.bitmap[idx] &= !(1 << bit);
        self.used_frames -= 1;

        // Otimização: se liberamos algo antes do next_free, atualizamos
        if idx < self.next_free {
            self.next_free = idx;
        }
    }

    /// Retorna estatísticas de uso de memória
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.used_frames,
            self.total_frames,
            self.total_frames - self.used_frames,
        )
    }
}
