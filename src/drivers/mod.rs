// (FASE2) src/drivers/mod.rs
//! # Kernel Driver Layer
//!
//! O módulo `drivers` é a camada que implementa a lógica específica de dispositivos,
//! traduzindo comandos de alto nível do kernel para I/O ports, MMIO ou chamadas de hardware.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Abstração de Hardware (Device Drivers):** Encapsula a complexidade de registradores (ex: UART, VGA) em APIs seguras.
//! - **Gerenciamento de Estado:** Mantém o estado global dos dispositivos (ex: Structs protegidas por `Mutex`).
//! - **Inicialização:** Fornece métodos `init` que devem ser chamados em ordens específicas pelo `core::entry`.
//!
//! ## 🏗️ Catálogo de Drivers (Sub-módulos)
//!
//! | Driver    | Responsabilidade | Estado Atual |
//! |-----------|------------------|--------------|
//! | `console` | Gerencia o Framebuffer gráfico como um terminal de texto (TTY). Suporta scroll, cores e wrapping. | **Alpha:** Scroll via memcpy (lento), sem suporte a Escape codes ANSI completos. |
//! | `pic`     | Controlador de Interrupções Legado (8259A). Mapeia IRQs 0-15 para vetores 32-47. | **Legado:** Essencial para boot, mas obsoleto em face do APIC. |
//! | `serial`  | Porta Serial (COM1/UART 16550). Saída primária de logs para debug. | **Estável:** Polling mode (bloqueante) para garantir entrega de logs. |
//! | `timer`   | Programmable Interval Timer (PIT 8254). Gera o heartbeat do sistema e contagem de uptime. | **Legado:** Limitado a ~1kHz preciso. Deve ser substituído por Local APIC Timer. |
//! | `video`   | Subsistema de vídeo primitivo (Framebuffer linear). Limpa tela e desenha pixels. | **Básico:** Apenas desenha pixels. Sem aceleração, sem double-buffering. |
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Isolamento de Concorrência:** Todos os drivers globais (`SERIAL1`, `CONSOLE`, `PICS`) são protegidos por `Mutex<T>`, garantindo acesso seguro em SMP.
//! - **Simplicidade:** Implementações "bare-bones" facilitam o entendimento e debug inicial.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Uso de Hardware Legado:** Depender de PIC e PIT limita a performance e escalabilidade (limite de 15 IRQs, precisão baixa).
//! - **Drivers Bloqueantes:** O driver serial usa *busy wait* (`while !empty`), o que pode travar o kernel se o hardware falhar.
//! - **Acoplamento Gráfico:** O `console` depende diretamente do `video`, e está rodando inteiramente na CPU (Software Rendering), o que consome ciclos de CPU preciosos.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Modernization)** Implementar driver **IO-APIC** e **Local APIC**.
//!   - *Motivo:* Suporte a Multicore real e vetores de interrupção > 15.
//! - [ ] **TODO: (Output)** Implementar um buffer circular (RingBuffer) para a Serial.
//!   - *Motivo:* Transformar o driver em *Interrupt-driven* para não gastar CPU esperando o byte ser enviado.
//! - [ ] **TODO: (Graphics)** Implementar Double Buffering no Console.
//!   - *Impacto:* Eliminar o "tearing" visual durante o scroll e acelerar o redesenho.
//! - [ ] **TODO: (Input)** Adicionar driver de Teclado (PS/2 inicialmente, USB XHCI futuro).
//!   - *Status:* Atualmente o sistema não tem input.

pub mod console; // Framebuffer Text Console
pub mod pic;
pub mod serial; // UART 16550 (Logs)
pub mod test;
pub mod timer; // PIT 8254 // 8259 PIC
pub mod video;

// Futuro:
// pub mod keyboard;
// pub mod pci;
