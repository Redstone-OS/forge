//! Physical Memory Manager (PMM) — Bitmap Frame Allocator
//! -----------------------------------------------------
//! Gerencia a alocação de frames físicos (4 KiB) através de um bitmap.
//! Projetado para ser simples, determinístico e adequado ao early-kernel do
//! Redstone OS. O bitmap é armazenado em memória física (colocado em uma região
//! "usable" grande no boot) e cada bit representa um frame: 0 = livre, 1 = usado.
//!
//! ### Contratos / Invariantes
//! - `BitmapFrameAllocator::init()` **deve** ser chamado cedo, com um `BootInfo` válido.
//! - O bitmap é colocado numa região Usable do mapa de memória; o init garante que
//!   a região escolhida não conflita com o kernel ou com o próprio bitmap.
//! - `FRAME_SIZE` é 4 KiB (constante); todas as contas de frames e alinhamentos usam esse valor.
//! - A estrutura é normalmente protegida por `FRAME_ALLOCATOR: Mutex<...>` para uso concorrente.
//!
//! ### Segurança / notas de `unsafe`
//! - `init()` faz conversões de ponteiro físico -> slice mutável; isso é `unsafe`.
//!   Garantimos que o endereço e tamanho escolhidos são válidos e alinhados a 8 bytes.
//! - Operações de marcação/limpeza do bitmap manipulam bits diretamente; qualquer corrupção
//!   do bitmap pode causar alocações inválidas. Teste em QEMU antes de rodar em hardware.
//!
//! ### Melhoria futura (TODO)
//! - Detectar e reportar double-free com logging mais agressivo.
//! - Suportar lock-free allocation fast-path para múltiplos CPUs.
//! - Compactar / remover frames reservados por dispositivos (MMIO) ao construir mapa.
//!

use crate::core::handoff::{BootInfo, MemoryType};
use crate::sync::Mutex;
use core::sync::atomic::{AtomicUsize, Ordering};

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
        if boot_info.memory_map_len == 0 {
            panic!("BootInfo não contém mapa de memória válido!");
        }

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;
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
            "PMM: Memória máxima: {:#x} ({} MB)",
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
            panic!("PMM: Não foi possível encontrar região Usable para o bitmap!");
        }

        crate::kinfo!(
            "PMM: Bitmap em {:#x}, {} frames",
            bitmap_phys_addr,
            total_frames
        );

        let bitmap_ptr = bitmap_phys_addr as *mut u64;
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
            "PMM: Inicializado. Total Frames: {}, Free: {}",
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
                let bit = entry.trailing_ones() as usize; // Retorna quantos 1s seguidos no final (LSB)
                                                          // Se entry não é MAX, trailing_ones < 64.

                let frame_idx = idx * 64 + bit;

                if frame_idx < self.total_frames {
                    // Marcar como usado
                    self.bitmap[idx] |= 1 << bit;
                    self.used_frames += 1;
                    self.next_free = idx;

                    return Some(PhysFrame {
                        addr: frame_idx as u64 * FRAME_SIZE as u64,
                    });
                }
            }
        }

        None // OOM (Out of Memory)
    }

    pub fn deallocate_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        // Verificar se já estava livre (Double Free)
        if (self.bitmap[idx] & (1 << bit)) == 0 {
            // Log warning? Panic? Por enquanto ignora.
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
}
