# Arquitetura do Kernel Forge

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Princípios de Design](#princípios-de-design)
- [Arquitetura de Módulos](#arquitetura-de-módulos)
- [Fluxo de Inicialização](#fluxo-de-inicialização)
- [Modelo de Execução](#modelo-de-execução)

---

## Visão Geral

O **Forge** é um kernel de sistema operacional de arquitetura **monolítica modular**, escrito em Rust. Ele opera em nível de privilégio máximo (Ring 0) e gerencia todos os recursos de hardware, fornecendo abstrações para aplicações de usuário.

Apesar de ser monolítico (todos os serviços rodam no mesmo espaço de endereçamento), o Forge enfatiza uma forte separação lógica entre subsistemas, inspirada em microkernels, para facilitar manutenção e estabilidade.

### Diagrama de Alto Nível

```mermaid
graph TB
    subgraph Userspace [Ring 3]
        App1[Shell / Init]
        App2[Serviços de Sistema]
    end

    subgraph KernelSpace [Ring 0 - Forge]
        SYSCALL[Interface de Syscalls]
        
        subgraph Core [Núcleo]
            SCHED[Scheduler]
            IPC[IPC]
            SYNC[Sincronização]
        end
        
        subgraph Resources [Gerenciamento]
            VMM[Virtual Memory]
            PMM[Physical Memory]
            VFS[Virtual File System]
        end
        
        subgraph HAL [Hardware Abstraction]
            ARCH[Arch (x86_64)]
            DRV[Drivers]
        end
    end

    Userspace --> |Syscall / Interrupt| SYSCALL
    SYSCALL --> Core
    Core --> Resources
    Resources --> HAL
    HAL --> Hardware
```

---

## Princípios de Design

1.  **Segurança em Primeiro Lugar**: Uso extensivo do sistema de tipos do Rust e Ownership para prevenir corrupção de memória e Data Races. Código `unsafe` é isolado e auditado.
2.  **Modularidade**: Componentes como Scheduler, VMM e VFS são fracamente acoplados. Implementações podem ser trocadas com impacto mínimo.
3.  **Assincronismo**: O kernel é projetado para lidar com eventos e interrupções de forma eficiente, minimizando latência.
4.  **KISS (Keep It Simple, Stupid)**: Preferência por implementações simples e legíveis sobre otimizações prematuras complexas.

---

## Arquitetura de Módulos

O código fonte está organizado em camadas hierárquicas em `src/`:

### 1. Camada de Hardware (HAL)

-   **`arch/`**: Código específico da arquitetura (x86_64). Controla registradores (CR3, CR0), tabelas globais (GDT, IDT), e interrupções.
-   **`drivers/`**: Drivers de dispositivos simples (Serial, Timer, Vídeo, Teclado).

### 2. Camada Central (Core)

-   **`core/`**: Inicialização do sistema, tratamento de pânico e recepção do BootInfo (handoff do bootloader).
-   **`entry.rs`**: Ponto de entrada Rust (`kernel_main`).
-   **`mm/`**: Gerenciador de Memória.
    -   `pmm.rs`: Alocador de quadros físicos (Physical Frame Allocator).
    -   `vmm.rs`: Gerenciamento de tabelas de páginas (Page Tables).
    -   `heap.rs`: Alocador dinâmico do Kernel (Heap Allocator).

### 3. Camada de Sistema (System)

-   **`sched/`**: Escalonador de processos e threads (Cooperativo/Preemptivo).
-   **`ipc/`**: Mecanismos de troca de mensagens entre processos.
-   **`sync/`**: Primitivas de sincronização (Mutex, Spinlock) para garantir integridade em ambiente multitarefa.
-   **`sys/`**: Definições de ABI, constantes de erro e tipos base.

### 4. Camada de Interface

-   **`syscall/`**: Handlers para chamadas de sistema vindas do userspace.
-   **`fs/`**: Virtual File System (VFS), permitindo montagem de diferentes sistemas de arquivos (Ext2, FAT32, RAMFS).

---

## Fluxo de Inicialização

O processo de boot do Forge segue uma sequência estrita:

1.  **Ignite Bootloader**: Carrega o kernel na memória, configura o modo Long Mode (64-bit), coleta informações de memória e hardware (ACPI), e salta para o kernel.
2.  **Entry Point (`_start` em `main.rs`)**:
    -   Função `naked` (Assembly puro).
    -   Salva o ponteiro `BootInfo` (passado em RDI).
    -   Configura uma nova pilha de kernel (Kernel Stack de 16KB).
    -   Habilita funcionalidades de CPU essenciais (SSE).
    -   Chama `kernel_core::entry::kernel_main`.
3.  **Kernel Main**:
    -   **Inicialização de Hardware**: Configura GDT, IDT e habilita interrupções.
    -   **Inicialização de Memória**: Inicializa o PMM e o Heap Allocator.
    -   **Drivers**: Inicializa drivers básicos (Serial, Vídeo).
    -   **Scheduler**: Cria o processo `init` (userspace).
    -   **Loop Principal**: Entra no loop de idle ou agenda o primeiro processo.

---

## Modelo de Execução

O Forge suporta **Multitarefa Preemptiva**.

-   **Privilégio**: O kernel roda em Ring 0. Aplicações rodam em Ring 3.
-   **Interrupções**: O kernel é "interrupt driven". Timers e dispositivos de I/O interrompem a CPU para processamento.
-   **Syscalls**: Aplicações solicitam serviços via instrução `syscall`.
