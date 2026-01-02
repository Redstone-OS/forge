# Documentação do Gerenciamento de Memória (`src/mm`)

> **Caminho**: `src/mm`  
> **Responsabilidade**: Gerenciar RAM Física (PMM), Endereçamento Virtual (VMM) e Alocação Dinâmica (Heap).  
> **Arquitetura**: Higher Half Direct Map (HHDM).

---

## 🏛️ Visão Geral da Arquitetura

O subsistema de memória é dividido em três camadas hierárquicas:

1.  **PMM (Physical Memory Manager)**:
    *   "Dono" da RAM crua.
    *   Gerencia `PhysFrame` (blocos de 4KB).
    *   Usa um **Bitmap Allocator** para rastrear frames livres/usados.
2.  **VMM (Virtual Memory Manager)**:
    *   Cria a ilusão de memória para processos.
    *   Gerencia Page Tables (PML4, PDPT, PD, PT).
    *   Abstração: `AddressSpace`.
3.  **Heap (Kernel Allocator)**:
    *   Fornece `Box`, `Vec`, `Arc` para o kernel.
    *   Implementa o trait `GlobalAlloc`.
    *   Backend: **Slab/Buddy Allocator** (ou Bump em estágios iniciais).

---

## 🗺️ Layout de Memória (HHDM)

O RedstoneOS utiliza a técnica **Higher Half Direct Map**.
Toda a RAM física disponível é mapeada linearmente em uma região fixa do kernel space.

*   **HHDM Base**: `0xFFFF_8000_0000_0000`
*   **Conversão**:
    *   Phys → Virt: `base + phys`
    *   Virt → Phys: `virt - base`

Isso permite que o kernel acesse *qualquer* endereço físico sem precisar alterar as tabelas de páginas (sem `kmap` temporário), aumentando drasticamente a performance de I/O e forks.

---

## 📂 Estrutura de Arquivos

### Core
| Arquivo | Função |
|:--------|:-------|
| `hhdm.rs` | Implementação do Direct Map e funções de conversão `phys_to_virt`. |
| `mod.rs` | Inicialização `unsafe fn init()` na ordem correta. |

### Subsistemas
| Diretório | Descrição |
|:----------|:----------|
| `pmm/` | Alocador de Frames físicos. Contém o `FRAME_ALLOCATOR` global. |
| `vmm/` | Manipulação de CR3 e Page Tables (map/unmap/flags). |
| `heap/` | Implementação do `#[global_allocator]`. |
| `cache/` | Page Cache (não implementado totalmente, para FS). |

---

## 🧩 Tipos Fortes (`addr/`)

Para evitar bugs de ponteiro, usamos tipos distintos que não se misturam aritmeticamente:

*   `PhysAddr(u64)`: Endereço físico real de hardware.
*   `VirtAddr(u64)`: Endereço virtual de software.

**Regra**: Você nunca pode desreferenciar um `PhysAddr` diretamente. Deve convertê-lo para `VirtAddr` via HHDM primeiro.

---

## 🚦 Ordem de Inicialização (Boot)

A função `init(boot_info)` deve ser cirúrgica:
1.  **VMM Init**: O bootloader passa a tabela de páginas atual. O VMM assume o controle.
2.  **HHDM Init**: Calcula onde a RAM está mapeada e valida se bate com o mapa de memória.
3.  **PMM Init**: Lê o Memory Map (E820/UEFI) e marca regiões usadas (kernel code, initrd) como ocupadas no bitmap.
4.  **Heap Init**: Aloca uma região inicial de páginas virtuais e entrega ao Slab Allocator.

---

## ⚠️ Segurança e Race Conditions

*   **PMM Lock**: O alocador de frames é protegido por um Spinlock. Em SMP, isso é um gargalo, então futuramente teremos *Per-CPU Page Lists*.
*   **TLB Flush**: Ao alterar mapeamentos (`unmap_page`), é crucial invalidar o TLB (`invlpg`) imediatamente para evitar que a CPU use traduções antigas.
