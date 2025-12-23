# Gerenciamento de Memória

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Mapa de Memória Física (PMM)](#mapa-de-memória-física-pmm)
- [Memória Virtual (VMM)](#memória-virtual-vmm)
- [Alocador de Heap](#alocador-de-heap)

---

## Visão Geral

O subsistema de memória do Forge (`forge::mm`) é responsável por gerenciar tanto a memória física (RAM) quanto a memória virtual. Ele garante que o kernel e os processos tenham acesso seguro e isolado aos recursos.

### Estrutura do Módulo

-   **`pmm.rs`**: Physical Memory Manager. Gerencia frames físicos.
-   **`vmm.rs`**: Virtual Memory Manager. Gerencia Page Tables (PML4, PDPT, PD, PT).
-   **`heap.rs`**: Kernel Heap Allocator. Permite uso de `Box`, `Vec`, `Arc`.

---

## Mapa de Memória Física (PMM)

O **Physical Memory Manager** rastreia quais páginas de memória física (Frames de 4KB) estão livres e quais estão ocupadas.

### Inicialização
O PMM é inicializado usando o `MemoryMap` fornecido pelo Ignite Bootloader (via UEFI). O mapa descreve regiões como:
-   `Usable`: Memória livre para uso.
-   `Reserved`: Reservado pelo hardware/bios.
-   `KernelCode`: Onde o código do kernel reside.
-   `BootloaderReclaim`: Dados do bootloader que podem ser reutilizados após o boot.

### Algoritmo
O Forge utiliza um **Bitmap Allocator** (ou similar) para rastrear frames livres.
-   **Alocação**: Encontra o primeiro bit livre no bitmap.
-   **Liberação**: Marca o bit correspondente como livre.

---

## Memória Virtual (VMM)

O **Virtual Memory Manager** implementa paginação (Paging) para x86_64 (4-level paging).

### Espaço de Endereçamento do Kernel
O kernel reside na metade superior da memória virtual (Higher Half Kernel), tipicamente acima de `0xFFFF_8000_0000_0000`. Isso garante que o kernel esteja sempre mapeado em todos os espaços de endereçamento de processos, facilitando syscalls e interrupções.

### Page Tables
O VMM abstrai a manipulação da estrutura de tabelas de páginas:
1.  **PML4** (Page Map Level 4)
2.  **PDPT** (Page Directory Pointer Table)
3.  **PD** (Page Directory)
4.  **PT** (Page Table)

```mermaid
graph LR
    CR3[Registrador CR3] --> PML4
    PML4 --> PDPT
    PDPT --> PD
    PD --> PT
    PT --> Frame[Frame Físico 4KB]
```

### Funcionalidades
-   `map_page(virt, phys, flags)`: Mapeia um endereço virtual a um físico.
-   `unmap_page(virt)`: Remove um mapeamento.
-   `translate(virt) -> phys`: Traduz um endereço (útil para DMA).

---

## Alocador de Heap

Para suportar estruturas de dados dinâmicas do Rust (`alloc` crate), o Forge implementa um Global Allocator.

### Implementação
Atualmente, o kernel utiliza um alocador baseado em **Linked List** ou **Bump Pointer** para o boot, evoluindo para um **Slab Allocator** ou **Buddy Allocator** para performance em tempo de execução.

### Exemplo de Uso
```rust
extern crate alloc;
use alloc::vec::Vec;

pub fn example() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    // Memória alocada dinamicamente no heap do kernel
}
```

### Handler de Erro
Se o kernel ficar sem memória (OOM), o `alloc_error_handler` é acionado, causando um *kernel panic* controlado para evitar corrupção de dados.
