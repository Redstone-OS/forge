# Documentação da Arquitetura HAL (`src/arch`)

> **Caminho**: `src/arch`  
> **Responsabilidade**: Hardware Abstraction Layer. Isolar o código genérico do kernel das especificidades da CPU (x86_64, aarch64, riscv64).

---

## 🏛️ A Camada de Abstração

O Forge é desenhado para ser portátil. Todo código fora de `src/arch` deve compilar e rodar independentemente da arquitetura. O `src/arch` age como o tradutor.

### Traits Principais (`traits/`)
O módulo define interfaces que cada plataforma deve implementar:
*   `CpuTrait`: Funções como `halt()`, `disable_interrupts()`, `current_core_id()`.
*   `MmuTrait`: Funções para manipular Page Tables (`map`, `unmap`).
*   `ContextTrait`: Salvar e restaurar registradores.

---

## 🖥️ Implementação x86_64 (`x86_64/`)

A principal plataforma suportada atualmente.

### 1. `cpu.rs` & `gdt.rs`
Configura a **Global Descriptor Table** (obrigatória em x86). Define segmentos de Código e Dados para Kernel e User (Ring 0 vs Ring 3). Configura o TSS (Task State Segment) para troca de stacks.

### 2. `idt.rs` & `interrupts.rs`
Configura a **Interrupt Descriptor Table**. Mapeia exceções da CPU (Page Fault, Div by Zero) e IRQs de hardware (Timer, Teclado) para funções Rust (`extern "x86-interrupt"`).
*   Reprograma o PIC (Legacy) ou configura APIC/IOAPIC (Moderno).

### 3. `syscall.rs`
Configura os MSRs (Model Specific Registers) `LSTAR`, `STAR`, `FMASK` para habilitar a instrução rápida `SYSCALL`.

### 4. `memory.rs`
Implementa a manipulação das tabelas de paginação de 4 níveis (PML4).

---

## 🔄 Portabilidade

Para suportar uma nova arquitetura (ex: RISC-V), o desenvolvedor deve:
1.  Criar `src/arch/riscv64/`.
2.  Implementar `CpuTrait` e outros contratos.
3.  Configurar o boot entry point.
4.  Exportar o novo módulo em `src/arch/mod.rs` condicionalmente (`#[cfg(target_arch = "riscv64")]`).

O resto do kernel (Memory Manager, Scheduler, FS) funcionará sem modificações, pois consomem a API de `src/arch`.
