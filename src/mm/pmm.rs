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
/// - `bitmap_ptr` e `bitmap_len` representam o bitmap; cada bit (LSB = bit 0) representa um frame.
/// - `total_frames` é o número total de frames representados no bitmap.
/// - `memory_base` é o endereço físico base considerado para frame index 0.
///   (por simplicidade aqui usamos base = 0; poderia ser ajustado para offsets)
///
/// IMPORTANTE: Usamos ponteiro raw em vez de slice para evitar código SSE
/// que o Rust gera ao manipular fat pointers (16 bytes).
pub struct BitmapFrameAllocator {
    /// Início da região de memória gerenciada pelo bitmap.
    memory_base: u64,
    /// Ponteiro para o bitmap onde cada bit representa um frame de 4KiB.
    /// 0 = Livre, 1 = Usado.
    bitmap_ptr: *mut u64,
    /// Tamanho do bitmap em u64s.
    bitmap_len: usize,
    /// Total de frames gerenciados.
    total_frames: usize,
    /// Dica para a próxima alocação (round-robin simples).
    next_free: usize,
    /// Estatística de uso.
    used_frames: usize,
}

// SAFETY: O PMM é protegido por Mutex, então Send/Sync são seguros
unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}

impl BitmapFrameAllocator {
    /// Cria um alocador vazio — usado para inicialização estática.
    const fn empty() -> Self {
        Self {
            memory_base: 0,
            bitmap_ptr: core::ptr::null_mut(),
            bitmap_len: 0,
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

        // 1. Calcular memória total usando acesso direto (evita código SSE do slice iterator)
        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;
        crate::ktrace!("(PMM) init: map_ptr={:p}, map_len={}", map_ptr, map_len);
        crate::ktrace!("(PMM) init: Calculando memória máxima...");

        // Verificar se o memory map está acessível
        if !crate::mm::addr::is_phys_accessible(boot_info.memory_map_addr) {
            crate::kerror!(
                "(PMM) ERRO: memory_map em {:#x} fora do identity map!",
                boot_info.memory_map_addr
            );
            panic!("Memory map inacessível!");
        }
        crate::ktrace!("(PMM) init: Memory map acessível, iniciando leitura...");
        crate::ktrace!(
            "(PMM) init: sizeof(MemoryMapEntry) = {} bytes",
            core::mem::size_of::<crate::core::handoff::MemoryMapEntry>()
        );

        let mut max_phys_addr = 0u64;

        // DEBUG: Tentar ler primeiro byte via inline assembly puro
        let addr = boot_info.memory_map_addr;
        crate::ktrace!("(PMM) DEBUG: Tentando ler byte em {:#x}...", addr);

        let first_byte: u64;
        core::arch::asm!(
            "movzx {0}, byte ptr [{1}]",
            out(reg) first_byte,
            in(reg) addr,
            options(nostack, preserves_flags, readonly)
        );
        crate::ktrace!("(PMM) DEBUG: Primeiro byte = {:#x}", first_byte as u8);
        crate::ktrace!("(PMM) Leitura OK! Continuando...");

        // Usar while loop com índice manual para evitar otimizações do compilador
        let entry_size = 24u64; // MemoryMapEntry = u64 + u64 + u32 + padding = 24 bytes
        let mut i = 0usize;

        while i < map_len {
            // Calcular endereço da entry atual
            let entry_addr = boot_info.memory_map_addr + (i as u64 * entry_size);

            // Ler campos via inline assembly (evita SSE)
            let base: u64;
            let len: u64;
            let typ_raw: u32;

            core::arch::asm!(
                "mov {0}, [{3}]",        // base = *entry_addr
                "mov {1}, [{3} + 8]",    // len = *(entry_addr + 8)
                "mov {2:e}, [{3} + 16]", // typ = *(entry_addr + 16) (32-bit)
                out(reg) base,
                out(reg) len,
                out(reg) typ_raw,
                in(reg) entry_addr,
                options(nostack, preserves_flags, readonly)
            );

            if i < 5 {
                crate::ktrace!(
                    "(PMM) region[{}]: base={:#x} len={:#x} typ={}",
                    i,
                    base,
                    len,
                    typ_raw
                );
            }

            // MemoryType::Usable = 1
            if typ_raw == 1 {
                let end = base + len;
                if end > max_phys_addr {
                    max_phys_addr = end;
                }
            }

            i += 1;
        }
        crate::ktrace!("(PMM) init: Iteração completa, {} regiões", map_len);

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

        let mut j = 0usize;
        while j < map_len {
            let entry_addr = boot_info.memory_map_addr + (j as u64 * entry_size);
            let base: u64;
            let len: u64;
            let typ_raw: u32;
            core::arch::asm!(
                "mov {0}, [{3}]",
                "mov {1}, [{3} + 8]",
                "mov {2:e}, [{3} + 16]",
                out(reg) base,
                out(reg) len,
                out(reg) typ_raw,
                in(reg) entry_addr,
                options(nostack, preserves_flags, readonly)
            );
            // MemoryType::Usable = 1
            if typ_raw == 1 {
                if len >= bitmap_total_size as u64 && len > best_region_size {
                    let region_end = base + len;
                    let aligned_start =
                        (region_end - bitmap_total_size as u64) & !(FRAME_SIZE as u64 - 1);
                    if aligned_start >= base {
                        bitmap_phys_addr = aligned_start;
                        best_region_size = len;
                    }
                }
            }
            j += 1;
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
        crate::ktrace!("(PMM) Convertendo phys_addr para virt...");
        let bitmap_ptr = crate::mm::addr::phys_to_virt::<u64>(bitmap_phys_addr);
        crate::ktrace!("(PMM) bitmap_ptr = {:p}", bitmap_ptr);

        // Atribuir ponteiro e tamanho diretamente (evita fat pointer = SSE)
        self.bitmap_ptr = bitmap_ptr;
        self.bitmap_len = bitmap_size_u64;
        crate::ktrace!(
            "(PMM) Bitmap configurado: ptr={:p}, len={}",
            self.bitmap_ptr,
            self.bitmap_len
        );

        // Preencher bitmap com 0xFFFFFFFFFFFFFFFF (tudo ocupado) usando assembly
        // para evitar instruções SSE geradas pelo fill()
        crate::ktrace!("(PMM) Zerando bitmap ({} u64s)...", bitmap_size_u64);
        let mut fill_idx = 0usize;
        let fill_value = u64::MAX;
        while fill_idx < bitmap_size_u64 {
            let ptr = bitmap_ptr.add(fill_idx);
            core::arch::asm!(
                "mov [{0}], {1}",
                in(reg) ptr,
                in(reg) fill_value,
                options(nostack, preserves_flags)
            );
            fill_idx += 1;
        }
        crate::ktrace!("(PMM) Bitmap preenchido OK");

        self.memory_base = 0;
        self.total_frames = total_frames;
        self.used_frames = total_frames;

        // 3. Liberar regiões usáveis (protegendo kernel, bitmap e primeiros 16MB)
        let kernel_end = boot_info.kernel_phys_addr + boot_info.kernel_size;
        let bitmap_end = bitmap_phys_addr + (bitmap_size_u64 * 8) as u64;
        const MIN_USABLE_ADDR: u64 = 0x1000000; // 16 MB

        let mut k = 0usize;
        while k < map_len {
            let entry_addr = boot_info.memory_map_addr + (k as u64 * entry_size);
            let base: u64;
            let len: u64;
            let typ_raw: u32;
            core::arch::asm!(
                "mov {0}, [{3}]",
                "mov {1}, [{3} + 8]",
                "mov {2:e}, [{3} + 16]",
                out(reg) base,
                out(reg) len,
                out(reg) typ_raw,
                in(reg) entry_addr,
                options(nostack, preserves_flags, readonly)
            );
            // MemoryType::Usable = 1
            if typ_raw == 1 {
                let start_frame = base / FRAME_SIZE as u64;
                let end_frame = (base + len) / FRAME_SIZE as u64;

                let mut frame_idx = start_frame;
                while frame_idx < end_frame {
                    let addr = frame_idx * FRAME_SIZE as u64;

                    if addr != 0
                        && addr >= MIN_USABLE_ADDR
                        && !(addr >= boot_info.kernel_phys_addr && addr < kernel_end)
                        && !(addr >= bitmap_phys_addr && addr < bitmap_end)
                        && frame_idx < total_frames as u64
                    {
                        self.deallocate_frame(frame_idx as usize);
                    }
                    frame_idx += 1;
                }
            }
            k += 1;
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

        let mut i = 0usize;
        while i < self.bitmap_len {
            let idx = (start_search + i) % self.bitmap_len;

            // Ler entry via ponteiro (evita indexação de slice que pode gerar SSE)
            let entry: u64;
            unsafe {
                let ptr = self.bitmap_ptr.add(idx);
                core::arch::asm!(
                    "mov {0}, [{1}]",
                    out(reg) entry,
                    in(reg) ptr,
                    options(nostack, preserves_flags, readonly)
                );
            }

            if entry != u64::MAX {
                // Se não está tudo cheio (todos 1s)
                // Encontrar o bit 0 (livre)
                let bit = entry.trailing_ones() as usize;

                let frame_idx = idx * 64 + bit;

                if frame_idx < self.total_frames {
                    // Marcar como usado
                    let new_value = entry | (1u64 << bit);
                    unsafe {
                        let ptr = self.bitmap_ptr.add(idx);
                        core::arch::asm!(
                            "mov [{0}], {1}",
                            in(reg) ptr,
                            in(reg) new_value,
                            options(nostack, preserves_flags)
                        );
                    }
                    self.used_frames += 1;
                    self.next_free = idx;

                    let addr = frame_idx as u64 * FRAME_SIZE as u64;
                    return Some(PhysFrame { addr });
                }
            }
            i += 1;
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

        // Ler valor atual via ponteiro
        let entry: u64;
        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            core::arch::asm!(
                "mov {0}, [{1}]",
                out(reg) entry,
                in(reg) ptr,
                options(nostack, preserves_flags, readonly)
            );
        }

        // Verificar se já estava livre (Double Free)
        if (entry & (1 << bit)) == 0 {
            crate::kwarn!(
                "(PMM) DOUBLE FREE! frame={} addr={:#x}",
                frame_idx,
                frame_idx as u64 * FRAME_SIZE as u64
            );
            return;
        }

        // Marcar como livre (0)
        let new_value = entry & !(1u64 << bit);
        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            core::arch::asm!(
                "mov [{0}], {1}",
                in(reg) ptr,
                in(reg) new_value,
                options(nostack, preserves_flags)
            );
        }
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
