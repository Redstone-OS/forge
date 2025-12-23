# Documentação do Kernel Forge

Bem-vindo à documentação oficial do **Forge**, o kernel do Redstone OS.

O Forge é um kernel monolítico moderno, escrito em Rust, focado em segurança, estabilidade e performance. Ele é projetado para tirar proveito das garantias de segurança de memória do Rust enquanto fornece uma arquitetura robusta para sistemas operacionais de propósito geral.

## 📚 Índice de Documentação

### 🏗️ Arquitetura e Design
- [Arquitetura Geral](ARQUITETURA.md): Visão macro do sistema, fluxo de boot e design de subsistemas.
- [Módulos do Sistema](ARQUITETURA.md#arquitetura-de-módulos): Detalhes sobre a organização do código fonte.

### 🧠 Subsistemas Principais
- [Gerenciamento de Memória](MEMORIA.md): Paging, Heap Allocators, e Physical Memory Manager (PMM).
- [Gerenciamento de Processos](PROCESSOS.md): Scheduling, multitarefa e threads.
- [Sistema de Arquivos](FILESYSTEM.md): VFS, InitRAMFS e drivers de armazenamento.
- [Drivers e Hardware](DRIVERS.md): Modelo de drivers e suporte a hardware.

### 🔌 Interfaces e API
- [System Calls](SYSCALLS.md): Interface binária entre userspace e kernel.
- [IPC (Inter-Process Communication)](IPC.md): Mecanismos de comunicação entre processos.

### 🛠️ Guia do Desenvolvedor
- [Compilação e Execução](BUILD.md): Como compilar e rodar o kernel.
- [Guia de Contribuição](CONTRIBUTING.md): Padrões de código e fluxo de desenvolvimento.

---

## 🚀 Status do Projeto

| Subsistema | Status | Notas |
|------------|--------|-------|
| **Boot** | ✅ Estável | Boot via Ignite (UEFI) |
| **Memória** | 🚧 Em Progresso | Paging básico e Heap implementados |
| **Interrupções** | 🚧 Em Progresso | IDT e APIC básicos |
| **Scheduler** | ❌ Planejado | Ainda não implementado |
| **Userspace** | ❌ Planejado | Ring 3 jumps pendentes |
| **VFS** | ❌ Planejado | Estrutura inicial |

## 🔗 Links Úteis

- [Repositório Principal](https://github.com/redstone-os/redstone)
- [Ignite Bootloader Documentation](../ignite/docs/index.md)
