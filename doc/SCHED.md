# Documentação do Módulo Scheduler (`src/sched`)

> **Caminho**: `src/sched`  
> **Responsabilidade**: Gerenciamento de Tarefas, Troca de Contexto e Política de Escalonamento da CPU.  
> **Modelo**: Multitarefa Preemptiva Round-Robin.  
> **Status**: Estável (Single Core), WIP (SMP/Load Balancing).

---

## 🏛️ Visão Geral

O agendador do RedstoneOS (Forge) é o componente que transforma o hardware estático em um sistema dinâmico, permitindo que múltiplos fluxos de execução (Tasks) compartilhem os recursos do processador.

Ao contrário de sistemas puramente cooperativos, o Forge utiliza **Preempção**: o timer do sistema interrompe a execução periodicamente (Timer Interrupt), dando ao kernel a chance de suspender a tarefa atual e eleger outra, garantindo responsividade mesmo se um processo travar em loop infinito.

---

## 📂 Estrutura de Arquivos

A organização reflete a separação entre a *entidade* (Task) e o *motor* (Core).

### 1. `src/sched/task/` (Entidades)
Define *o que* é agendado.
| Arquivo | Descrição Técnica |
|:--------|:------------------|
| `entity.rs` | Struct `Task` principal. Contém PID, Pilhas, Espaço de Endereçamento e Handles. |
| `context.rs`| Struct `Context`, que salva os registradores callee-saved (RBX, RBP, R12-R15) durante o switch. |
| `state.rs` | Enum `TaskState` (Running, Ready, Blocked, Zombie). |
| `lifecycle.rs` | Lógica de criação e destruição (gerenciamento de memória e Zumbis). |

### 2. `src/sched/core/` (O Motor)
Define *como* e *quando* agendar.
| Arquivo | Descrição Técnica |
|:--------|:------------------|
| `scheduler.rs` | Loop principal (`schedule()`), `yield_now()`, `sleep_current()`. |
| `runqueue.rs` | Fila de tarefas prontas (`Ready`). Atualmente uma `VecDeque` protegida por Spinlock. |
| `switch.rs` | Camada de abstração sobre o assembly `context_switch`. |
| `idle.rs` | A "Idle Task" - loop infinito que executa `HLT` quando não há nada para rodar (economiza energia). |

### 3. `src/sched/exec/` (Carregador)
| Arquivo | Descrição Técnica |
|:--------|:------------------|
| `loader.rs` | Parser ELF. Lê binários, mapeia segmentos em memória e prepara a `Task` inicial. |

---

## 🔄 Ciclo de Vida do Agendamento (The Loop)

O coração do sistema é a função `schedule()`, acionada voluntariamente (`yield`) ou involuntariamente (Interrupção de Timer).

```mermaid
graph TD
    Running((Running)) -->|Yield / Timeout| Schedule{schedule()}
    Schedule -->|Pick Next| ReadyQueue[RunQueue]
    
    ReadyQueue -->|Next Task| Switch[Context Switch]
    Switch -->|Load Context| Running
    
    Running -->|Sleep / Wait| Blocked((Blocked))
    Blocked -->|Event / Wakeup| ReadyQueue
    
    Running -->|Exit| Zombie((Zombie))
    Zombie -->|Waitpid| Dead[Destroyed]
```

### Context Switch (Troca de Contexto)
A mágica acontece em Assembly (`arch/x86_64/switch.s`).
1.  **Salvar**: O kernel empilha (`PUSH`) os registradores que a ABI exige preservar (RBP, RBX, R12-R15) na pilha da tarefa antiga.
2.  **Trocar SP**: O ponteiro da pilha (`RSP`) é salvo na struct `Task` antiga e o `RSP` da nova `Task` é carregado.
3.  **Restaurar**: O kernel desempilha (`POP`) os registradores da nova pilha.
4.  **Retornar**: Ao executar `RET`, a CPU "retorna" para onde a nova tarefa parou na última vez.

---

## ⚙️ Configurações (`config.rs`)

Parâmetros ajustáveis para tuning do sistema.

| Constante | Valor Padrão | Descrição |
|:----------|:-------------|:----------|
| `DEFAULT_QUANTUM` | `10` ticks | Tempo máximo que uma tarefa roda antes de sofrer preempção. |
| `KERNEL_STACK_SIZE`| `64 KB` | Tamanho da pilha privilegiada (Ring 0). |
| `USER_STACK_SIZE` | `2 MB` | Tamanho da pilha do usuário (Ring 3). |
| `PRIORITY_DEFAULT`| `128` | Prioridade base. (Otimizações de prioridade ainda WIP). |

---

## 🛠️ Guia de API Interna (Kernel Dev)

Se você está escrevendo um driver ou uma syscall, estas são as funções que você usará:

### 1. `sched::yield_now()`
Abraça a cooperatividade. Diz ao scheduler: "Posso parar agora se alguém precisar da CPU". Útil em loops longos de kernel.

### 2. `sched::spawn(path)`
Cria um novo processo a partir de um arquivo executável.
- Aloca nova `Task`.
- Cria novo `AddressSpace` (Page Tables).
- Carrega ELF.
- Coloca na `RunQueue`.

### 3. `sched::exit_current(code)`
Suicídio do processo. Transforma a tarefa em Zombie e nunca retorna.

### 4. `sched::core::current()`
Retorna uma referência à Tarefa que está rodando **agora** neste núcleo. Essencial para acessar handles, arquivos abertos e identidade.

---

## ⚠️ Race Conditions e SMP

A implementação atual utiliza um **Global Scheduler Lock** (ou locks finos na RunQueue) para proteger as listas.
*   **Perigo**: Nunca chame `schedule()` segurando um Spinlock que outra CPU possa precisar para agendar. Isso causa Deadlock instantâneo do sistema inteiro.
*   **Interrupts**: O Context Switch desabilita interrupções brevemente para garantir atomicidade da troca de `RSP`.

---

## 🔮 Roadmap (Futuro)

1.  **Multicore (SMP)**: Migrar de uma RunQueue Global para RunQueues Per-Core (escalabilidade linear).
2.  **Work Stealing**: Permitir que núcleos ociosos "roubem" tarefas de núcleos sobrecarregados.
3.  **Real-Time**: Implementar classes de agendamento `FIFO` e `RR` com prioridade estrita para drivers de áudio/controle.
