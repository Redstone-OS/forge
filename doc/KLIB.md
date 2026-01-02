# Documentação da Kernel Library (`src/klib`)

> **Caminho**: `src/klib`  
> **Responsabilidade**: Fornecer estruturas de dados e algoritmos utilitários `no_std` para o kernel.

---

## 🏛️ Visão Geral

Como o kernel opera em ambiente `no_std` (sem a biblioteca padrão do Rust), não temos acesso a muitas facilidades. O `klib` preenche essa lacuna fornecendo implementações otimizadas para uso em Ring 0.

Diferente da `alloc` (que fornece `Vec`, `BTreeMap`), o `klib` foca em estruturas de baixo nível ou intrínsecas de SO.

---

## 📂 Estrutura de Arquivos

| Arquivo/Módulo | Propósito |
|:---------------|:----------|
| `bitmap.rs` | Manipulação eficiente de arrays de bits. Usado pelo PMM para rastrear frames livres. |
| `bitflags.rs` | Macros para criar enums de flags type-safe (ex: permissões de página R/W/X). |
| `list/` | Listas intrusivas (Linked Lists onde os nós fazem parte da struct de dados). Essencial para o Scheduler. |
| `tree/` | Árvores (RBTree, AVL) para indexação rápida (ex: VMA lookup). |
| `align.rs` | Funções matemáticas para alinhamento de memória (`align_up(4000, 4096) -> 4096`). |
| `mem_funcs.rs` | Otimizações de `memcpy`, `memset`, `memcmp` (frequentemente em Assembly). |

---

## 🛠️ Utilitários Principais

### `Bitmap`
Uma estrutura que gerencia um array de `u64` como um campo contínuo de bits.
*   **Uso**: PMM (Physical Memory Manager).
*   **Feature**: Busca O(N) otimizada para encontrar o primeiro bit zero (primeiro frame livre).

### `align_up / align_down`
Crucial para paginação.
*   Exemplo: Se você pede 100 bytes de memória, mas a página é 4096, você precisa arredondar para 4096.

### `Intrusive Lists` (`list/`)
Diferente do `Vec` (que aloca no heap), listas intrusivas usam ponteiros dentro da própria estrutura `Task`.
*   **Vantagem**: Nenhuma alocação de memória para adicionar/remover da RunQueue. `enqueue` e `dequeue` são O(1) e nunca falham por OOM.

---

## ⚠️ Convenções

*   **No Panic**: Funções no `klib` devem evitar `panic!` a todo custo. Retorne `Result` ou `Option`.
*   **Performance**: Este código é "hot path". Otimizações são bem-vindas (ex: usar instruções de bit manipulation da CPU).
