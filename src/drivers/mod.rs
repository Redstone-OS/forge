//! # Redstone Driver Model (RDM)
//!
//! Arquitetura e ponto de entrada para o gerenciamento de hardware do RedstoneOS.
//! Projeto modular, orientado a camadas, com foco em inicialização previsível,
//! fácil extensão e recuperação em execução.
//!
//! ### Visão geral
//! - O RDM centraliza descoberta, pareamento (probe) e ciclo de vida dos drivers.
//! - Cada submódulo corresponde a uma camada ou família de drivers.
//! - A ordem de inicialização é crítica: base -> system -> comm -> bus -> drivers.
//!
//! ### Camadas
//! 1. **base/** — Gerência do modelo: `DriverManager`, monitoramento, energia e
//!    mecanismos de recuperação.
//! 2. **system/** — Serviços essenciais: timers, IRQs, rotinas de interrupção.
//! 3. **comm/** — Canal de comunicação primário (console serial / logs) — deve estar
//!    operacional cedo para depuração durante o boot.
//! 4. **bus/** — Abstração dos barramentos físicos (PCI, USB, ACPI). É responsável
//!    por descobrir dispositivos e criar `Device`es para o DriverManager.
//! 5. **drivers funcionais/** — Implementam `probe`/`attach`/`init` para classes:
//!    `storage`, `display`, `network`, `input`, `sound`, etc.
//!
//! ### Fluxo de inicialização (resumido)
//! 1. `base::init()` — prepara estruturas internas e registradores do RDM.
//! 2. `system::init()` — habilita timers e IRQs.
//! 3. `comm::init()` — abre console/serial para logs de boot.
//! 4. `bus::init()` — escaneia e instancia dispositivos físicos.
//! 5. Inicializa drivers funcionais (cada driver chama `probe`/registro no DriverManager).
//!
//! ### Boas práticas ao adicionar um driver
//! - Expor `pub fn init()` no módulo do driver para registro inicial (opcional).
//! - Implementar `probe(device)` separado da lógica de `init()` para facilitar hot-plug.
//! - Evitar efeitos colaterais globais durante `init()` — preferir registro e callbacks.
//!
//! ### Notas de confiabilidade
//! - Logs de inicialização são essenciais — mantenha mensagens concisas e idempotentes.
//! - O RDM deve tolerar falha parcial: um driver com erro não pode abortar todo o subsistema.
//! - Ordem e dependências entre drivers devem estar documentadas no próprio módulo.
//!
//! ### Resumo (rápido)
//! - Objetivo: descoberta robusta, pareamento automático e runtime resiliente.
//! - Inicialize na ordem indicada e use `probe` para anexar drivers a dispositivos encontrados.
//! - Mantenha `init()` leve; delegue trabalho pesado para threads/tasks pós-boot.

/// --- Submódulos core: infraestrutura e orquestração ---
pub mod base; // Gerenciamento e resiliência (DriverManager, power, watchdog)
pub mod bus;
pub mod system; // Timers, IRQs, tratamento de exceções // PCI, USB, ACPI — descoberta e abstração de barramentos

/// --- Submódulos funcionais: drivers por categoria ---
pub mod comm; // Console / Serial / Logging
pub mod display; // Vídeo, framebuffers, GPUs
pub mod input; // Teclado, mouse, touch
pub mod network; // Ethernet, Wi-Fi
pub mod sound; // Áudio (planejado / drivers)
pub mod storage; // Storage (discos, SSD, ramdisks)

/// Orquestra a inicialização de todo o subsistema de drivers do RedstoneOS.
/// Esta função deve ser chamada cedo no boot (entry.rs) para garantir que
/// o hardware esteja disponível para os serviços do kernel.
pub fn init(fb_info: crate::core::boot::handoff::FramebufferInfo) {
    // 1. Inicializa o Modelo Base (RDM)
    crate::kinfo!("(Drivers) Iniciando Orquestração de Hardware...");
    base::init();

    // 2. Inicializa o Suporte de Vida (Timers e IRQs)
    crate::kinfo!("(Drivers) Inicializando Suporte de Vida...");
    system::init();

    // 3. Inicializa Barramentos (estruturas, não escaneia ainda)
    crate::kinfo!("(Drivers) Inicializando Barramentos...");
    bus::init();

    // 4. Registra os Drivers Funcionais no RDM ANTES do scan
    // Assim quando dispositivos forem registrados, já existe driver para matching
    crate::kinfo!("(Drivers) Registrando Drivers...");

    // Storage drivers (virtio-blk, nvme, ahci, ata, ramdisk)
    storage::init();

    // Display drivers
    display::init(fb_info);

    // Network drivers
    network::init();

    // Input drivers
    input::init();

    // Sound drivers
    sound::init();

    // Communication drivers
    comm::init();

    // 5. Escaneia hardware e registra dispositivos no RDS
    // O matching com drivers acontece automaticamente aqui
    crate::kinfo!("(Drivers) Escaneando Hardware...");
    let devices = bus::scan_all();
    crate::kdebug!("(Drivers) Dispositivos encontrados:", devices.len());

    // Registra todos os dispositivos no RDS
    // O RDS fará probe() em cada driver registrado
    for dev in devices {
        base::register_device(dev);
    }

    crate::kinfo!("(Drivers) Sistema de hardware operacional.");
}
