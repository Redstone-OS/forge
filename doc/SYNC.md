# Documentação do Módulo Sync (`src/sync`)

> **Caminho**: `src/sync`  
> **Responsabilidade**: Primitivas de controle de concorrência e gerenciamento de estado compartilhado entre CPUs/Tasks.  
> **Nível**: Ring 0 (Kernel).

---

## 🏛️ Visão Geral

O módulo `sync` fornece as ferramentas fundamentais para garantir *Thread Safety* no kernel. Como o RedstoneOS é um kernel preemptivo e SMP (Symmetric Multi-Processing), o acesso a estruturas globais deve ser estritamente controlado.

Implementamos três categorias principais de bloqueio:

| Primitiva | Comportamento | Uso Ideal | Contexto Perigoso |
|:----------|:--------------|:----------|:------------------|
| **Spinlock** | Busy-wait (gira na CPU) | Seções críticas **muito curtas** (< 1µs). | Nunca usar se a seção for longa (trava a CPU inteira). |
| **Mutex** | Sleep (cede a CPU) | Seções longas ou que envolvem I/O. | **Proibido** em Interrupt Handlers (pode causar Deadlock ou crash no scheduler). |
| **RCU** | Lock-free Reads | Estruturas muito lidas e pouco escritas. | Não serve para consistência forte imediata. |

---

## 📂 Estrutura de Arquivos

| Diretório | Arquivo Principal | Descrição Técnica |
|:----------|:------------------|:------------------|
| `spinlock/` | `spinlock.rs` | Bloqueio atômico com desabilitação de interrupções (`CLI/STI`). |
| `mutex/` | `mutex.rs` | Bloqueio com fila de espera (atualmente fallback para spinning enquanto não integra com scheduler). |
| `rcu/` | `rcu.rs` | Read-Copy-Update baseado em contagem de referências (`Arc`). |
| `atomic/` | `atomic.rs` | Wrappers de conveniência sobre `core::sync::atomic`. |
| `rwlock/` | `rwlock.rs` | Leitura simultânea (N), escrita exclusiva (1). |
| `semaphore/`| `semaphore.rs` | Controle de recursos contáveis. |

---

## 🔧 Detalhes de Implementação

### 1. Spinlock (`src/sync/spinlock`)

Nossa implementação de `Spinlock<T>` é **Interrupt-Safe**.

*   **Entrada (`lock`)**:
    1.  Salva o estado atual das interrupções (`RFLAGS.IF`).
    2.  Desabilita interrupções (`cli`). Isso impede que o handler de interrupção tente pegar o mesmo lock (prevenindo deadlock recursivo na mesma CPU).
    3.  Executa `compare_exchange` atômico em loop (`hint::spin_loop()`).
*   **Saída (`drop`)**:
    1.  Libera o lock atômico.
    2.  Restaura as interrupções se estavam habilitadas anteriormente.

```rust
// Exemplo de uso
static DATA: Spinlock<Vec<u32>> = Spinlock::new(Vec::new());

fn handler() {
    // Interrupções OFF aqui dentro
    let mut guard = DATA.lock();
    guard.push(1);
} // Interrupções restauradas
```

### 2. Mutex (`src/sync/mutex`)

Atualmente, o `Mutex` está em estágio de transição.
*   **Status Atual**: Comporta-se similar a um Spinlock (faz busy-wait).
*   **Meta (TODO)**: Integrar com a fila de espera do Scheduler para colocar a thread atual para dormir (`Block`) e acordá-la (`Wake`) quando o lock for liberado.

Possui proteção contra *Priority Inversion* trivial (FIFO) e deadlock detection básico via `owner` ID.

### 3. RCU (Read-Copy-Update) (`src/sync/rcu`)

Implementação simplificada focada em **segurança de memória**.
*   **Leitores (`read`)**:
    *   Lock-free (apenas incrementa um contador atômico `Arc`).
    *   Rápido e não bloqueia escritores.
*   **Escritores (`update`)**:
    *   Cria uma **cópia** dos dados.
    *   Modifica a cópia.
    *   Troca o ponteiro global atomicamente.
    *   Aguarda que os leitores antigos terminem (via `Arc::decrement`).

Ideal para listas de processos, tabelas de descritores de arquivo ou configurações globais.

---

## ⚠️ Regras de Ouro (Kernel Safety)

1.  **Interrupções**: Se você está em um tratador de interrupção (IRQ), **USE SPINLOCK**. Nunca use Mutex. Mutexes podem tentar dormir, e não existe "dormir" dentro de uma interrupção de hardware (panic certo).
2.  **Ordem de Aquisição**: Sempre adquira locks na mesma ordem global para evitar Deadlocks (ABBA).
3.  **Hold Time**: Segure Spinlocks pelo menor tempo possível. Milhares de ciclos desperdiçados em spinlock afetam a performance global do sistema drasticamente.
