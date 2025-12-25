//! # Kernel Library (KLib)
//!
//! A `klib` é uma coleção de utilitários de baixo nível, agnósticos de arquitetura,
//! que complementam a `core` library do Rust para ambientes bare-metal.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Algoritmos Básicos:** Bitmaps, Listas, Alinhamento de memória.
//! - **Runtime functions:** Implementações de `memcpy`, `memset` (necessárias quando não linkamos com libc).
//! - **Helpers:** Funções `const` para cálculo de endereços (ex: `align_up`).
//!
//! ## 🏗️ Arquitetura dos Módulos
//!
//! | Módulo      | Responsabilidade | Estado Atual |
//! |-------------|------------------|--------------|
//! | `bitmap`    | Gerenciamento de bits (usado pelo PMM para rastrear frames). | **Funcional:** Busca linear simples (O(N)). |
//! | `mem_funcs` | Implementação de `memset/memcpy` em Rust. (Desabilitado) | **Crítico:** Implementação manual lenta e possivelmente instável. |
//! | `util`      | Funções de alinhamento (`align_up`, `align_down`). | **Estável:** Primitivas `const fn` eficientes. |
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Independência:** Não depende de alocação (Heap) ou `lock` (Concurrency), seguro para uso em estágios iniciais de boot.
//! - **Simplicidade:** O `Bitmap` opera sobre slices `&mut [u64]`, permitindo alocação estática ou dinâmica.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Performance do Bitmap:** A função `find_first` faz um scan linear bit a bit. Para bitmaps grandes (ex: 4GB RAM = 128KB bitmap), isso é lento.
//! - **Memória Volátil em `mem_funcs`:** As funções de memória usam `read/write_volatile`. Isso impede otimizações do compilador (auto-vectorization) e torna `memcpy` ordens de magnitude mais lento que o ideal para RAM normal.
//! - **Estabilidade:** `mem_funcs` está comentado no `mod.rs` indicando problemas de crash ou conflito com `compiler_builtins`.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Performance)** Otimizar `Bitmap::find_first` usando instruções intrínsecas (`ctz`, `lzcnt`).
//!   - *Ganho:* Reduzir custo de alocação de O(N) para O(N/64) ou O(1) com hints.
//! - [ ] **TODO: (Arch)** Reimplementar `memcpy/memset` em Assembly (ASM) otimizado.
//!   - *Motivo:* Rust seguro (mesmo com pointers) é difícil de bater implementações "hand-tuned" em ASM usando registros SSE/AVX.
//! - [ ] **TODO: (Safety)** Separar `memcpy` (RAM) de `mmio_memcpy` (Device).
//!   - *Risco:* Usar `volatile` para mover dados de processo é desperdício. Usar `memcpy` normal em MMIO é bug (caching/reordering).

pub mod bitmap;
pub mod test;
// pub mod mem_funcs; // TEMPORARIAMENTE DESABILITADO - causou crash

/// Alinha um endereço para cima.
///
/// # Exemplo
/// `align_up(10, 4) -> 12`
#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Alinha um endereço para baixo.
///
/// # Exemplo
/// `align_down(10, 4) -> 8`
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Verifica se um endereço está alinhado.
#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    (addr & (align - 1)) == 0
}
