//! # x86_64 Hardware Abstraction Implementation
//!
//! Este módulo contém a implementação concreta da HAL para processadores Intel e AMD de 64 bits.
//! Ele segue o padrão "Long Mode" (IA-32e) e não suporta modo de compatibilidade de legado (32-bit protected/real mode)
//! exceto durante o bootstrap inicial vindo do `Ignite`.
//!
//! ## 🏗️ Sub-Módulos e Responsabilidades
//!
//! | Módulo       | Responsabilidade |
//! |--------------|------------------|
//! | `cpu`        | Implementa `CpuOps`, controle de MSRs, inicialização SSE/FPU, features de CPUID. |
//! | `gdt`        | Global Descriptor Table (Segmentação). Configura CS/DS para Kernel (Ring 0) e Userspace (Ring 3). |
//! | `idt`        | Interrupt Descriptor Table. Vetor de interrupções, exceções de CPU e mapeamento de Syscalls. |
//! | `interrupts` | Handlers de alto nível (Rust) e stubs assembly (`naked`) para tratamento de exceções. |
//! | `memory`     | Utilitários de paginação específicos de x86 (CR3, Page Tables). |
//! | `ports`      | Acesso legado a IO Ports (`inb`, `outb`), usado para Serial, PIC e PS/2. |
//!
//! ## ⚙️ Fluxo de Inicialização
//! 1. `gdt::init()`: Configura segmentos e TSS (Task State Segment) para ter stack segura em interrupções.
//! 2. `idt::init()`: Registra handlers de exceção (Page Fault, GPF, Double Fault) e remapeia PIC (legacy).
//! 3. `cpu::init_sse()`: Habilita FPU/SSE para evitar `#UD` em operações otimizadas do Rust (`memcpy`).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Assembler Inline Seguro:** O uso de `asm!` com constraints precisas (`nomem`, `preserves_flags`) evita corrupção sutil de estado.
//! - **Tratamento de Exceções Robusto:** O uso de IST (Interrupt Stack Tables) no TSS (se configurado) previne Double Faults por stack overflow.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Dependência do PIC 8259:** O código ainda usa o PIC legado reprogramado. Sistemas modernos devem usar APIC/x2APIC.
//!   - *Risco:* Performance ruim em multicore e latência de interrupção maior.
//! - **Context Switch Hardcoded:** O chaveamento de tarefas (`switch.s`) e syscalls (`syscall.s`) estão muito amarrados a convenções específicas.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Modernization)** Implementar Driver Local APIC e I/O APIC.
//!   - *Motivo:* Desativar o PIC 8259 legado. APIC é mandatório para SMP (Multicore).
//! - [ ] **TODO: (Security)** Implementar `KASLR` (Kernel Address Space Layout Randomization).
//!   - *Motivo:* Atualmente o kernel carrega em endereço fixo, o que facilita exploits ROP.
//! - [ ] **TODO: (Feature)** Habilitar `XSAVE`/`XRSTOR` para salvar estado de registradores extendidos (AVX/AVX-512).
//!   - *Motivo:* Sem isso, threads que usam vetorização (AVX) vão corromper o estado umas das outras.
//! - [ ] **TODO: (Cleanup)** Mover `syscall.s` e `switch.s` para arquivos `.S` separados com build.rs, ou usar `global_asm!` estruturado.

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod ports;

// Re-exporta a implementação concreta de CPU para uso genérico
pub use cpu::CpuidResult;
pub use cpu::X64Cpu as Cpu;

// Incluir Assembly do Handler de Syscall
core::arch::global_asm!(include_str!("syscall.s"));
