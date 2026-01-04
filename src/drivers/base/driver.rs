//! # Interface de Driver (Driver Layer)
//!
//! Este arquivo define o contrato que todo software de controle de hardware deve assinar.
//! O Redstone Driver Model utiliza polimorfismo para interagir com diferentes tipos
//! de hardware de forma transparente e segura.
//!
//! ## Funções do Driver:
//! - **`probe`**: O driver inspeciona o `Device` e decide se consegue controlá-lo.
//!   (Ex: O driver xHCI verifica se o dispositivo PCI tem a classe USB 3.0).
//! - **`remove`**: Chamado para liberar recursos e parar o hardware de forma limpa.
//! - **`suspend/resume`**: Gerencia a transição entre estados de energia.
//! - **`shutdown`**: Preparação crítica para desligamento do sistema.

use super::device::Device;

/// Tipo funcional do dispositivo (Classificação Geral)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Storage,    // Dispositivos de bloco (Drives, USB Sticks)
    Input,      // Teclado, Mouse, Sensores
    Display,    // Framebuffer, GPU
    Network,    // Ethernet, WiFi
    Serial,     // UART, COM, USB-Serial
    Audio,      // CODECs, Mixers
    Bus,        // Hubs, Bridges, Host Controllers (Ex: xHCI)
    Timer,      // HPET, PIT, RTC
    Controller, // DMA, Interrupt Controllers, IOMMU
    Security,   // TPM, Hardware RNG
    Generic,    // Miscelânea
    Unknown,
}

/// Erro retornado por operações de driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    NotSupported,     // Hardware não suportado por este driver
    InitFailed,       // Falha crítica na inicialização dos registradores
    BusError,         // Erro de comunicação no barramento (Timeout/NACK)
    DeviceError,      // O hardware reportou um erro interno
    Timeout,          // O hardware parou de responder
    InvalidParameter, // Configuração inválida sugerida pelo kernel
    NoMemory,         // Falha ao alocar DMA ou DMA Buffers
    AccessDenied,     // Falha de permissão (ex: BIOS lock)
    ResourceConflict, // O recurso (IRQ/Porta) já está em uso
}

/// Trait fundamental para todos os drivers do RedstoneOS.
/// Implementar esta trait torna o driver um cidadão de primeira classe no RDM.
pub trait Driver: Send + Sync {
    /// Nome identificador único do driver (ex: "virtio-blk-driver").
    fn name(&self) -> &'static str;

    /// Tipo funcional primário que este driver atende.
    fn device_type(&self) -> DeviceType;

    /// Tenta assumir o controle de um dispositivo.
    /// Chamado pelo `DriverManager` durante a descoberta de novo hardware.
    /// Deve realizar as seguintes tarefas:
    /// 1. Verificar IDs de hardware (Vendor/Device).
    /// 2. Reservar recursos (IRQ, MMIO).
    /// 3. Inicializar registradores base.
    /// 4. Criar interfaces de alto nível (ex: registrar no VFS).
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;

    /// Desfaz as operações do `probe`.
    /// Deve garantir que o hardware pare de gerar interrupções e usar DMA.
    fn remove(&self, dev: &mut Device) -> Result<(), DriverError>;

    /// Gerenciamento de Energia: Reduz o consumo do hardware.
    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }

    /// Gerenciamento de Energia: Restaura o estado operacional pleno.
    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }

    /// Chamado durante o power-off ou reboot.
    /// Deve colocar o hardware em um estado seguro/neutralizado.
    fn shutdown(&self, _dev: &mut Device) {
        // Opcional
    }

    /// Retorna flags de capacidades específicas do driver.
    fn capabilities(&self) -> u64 {
        0
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Matrizes de IDs (Device ID Table):
//    - Inspirado no Linux, permitir que o driver retorne uma lista de IDs
//      suportados (PCI Vendor/Device) para que o `DriverManager` faça o match
//      sem precisar chamar o `probe` em drivers incompatíveis.
//
// 2. Probing Assíncrono:
//    - Permitir que o `probe` seja executado em threads separadas para não
//      travar o boot com dispositivos lentos (ex: Inicialização de discos magnéticos).
//
// 3. Suporte a Firmwares (Microcode):
//    - Criar uma interface para que o driver solicite ao kernel o carregamento
//      de blobs de firmware antes do `probe`.
//
// 4. Hot-reloading de Drivers:
//    - Capacidade de descarregar e carregar drivers atualizados sem reiniciar o
//      sistema, migrando os objetos `Device` dinamicamente.
//
// 5. Estatísticas de Vivas (Liveness Hooks):
//    - Garantir que o driver forneça um método de "Pulsar" (Heartbeat) para o
//      `MonitorManager` confirmar que o driver não entrou em loop infinito.
