//! # Interface de Driver (Driver Trait)
//!
//! Este arquivo define o **contrato fundamental** que todo driver do RedstoneOS
//! deve implementar. A trait `Driver` é a "carteira de motorista" que permite
//! que um módulo de código assuma controle de um hardware.
//!
//! ## Responsabilidades do Driver:
//! - **Identificação**: Declarar nome e tipo funcional
//! - **Probe**: Verificar se consegue controlar um dispositivo específico
//! - **Lifecycle**: Gerenciar inicialização, suspensão e desligamento
//! - **Compatibilidade**: Declarar versão de ABI para hot-reload
//!
//! ## Filosofia:
//! O driver é um "inquilino" no condomínio do kernel. Ele não aloca memória
//! diretamente, não configura IRQs sozinho - tudo passa pela Base (síndico).
//!
//! ## Exemplo de Implementação:
//! ```rust
//! impl Driver for MeuDriver {
//!     fn name(&self) -> &'static str { "meu-driver" }
//!     fn device_type(&self) -> DeviceType { DeviceType::Network }
//!     fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
//!         // Verificar hardware, inicializar, etc
//!         Ok(())
//!     }
//!     fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
//!         // Limpar recursos
//!         Ok(())
//!     }
//! }
//! ```

use super::device::Device;

// =============================================================================
// TIPOS DE DISPOSITIVO
// =============================================================================

/// Classificação funcional de dispositivos.
///
/// Define a "categoria" do hardware. Um mesmo dispositivo físico pode ser
/// acessado via diferentes barramentos (USB, PCI), mas sua função é a mesma.
/// Esta classificação permite que o VFS e outros subsistemas encontrem
/// dispositivos por função, não por conexão física.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Dispositivos de armazenamento em bloco (SSDs, HDDs, Pendrives)
    /// Usados pelo VFS para montar sistemas de arquivos
    Storage,

    /// Dispositivos de entrada (Teclado, Mouse, Touchpad, Joystick)
    /// Geram InputEvents para o gerenciador de janelas
    Input,

    /// Dispositivos de saída visual (GPU, Framebuffer)
    /// Controlam o que aparece na tela
    Display,

    /// Dispositivos de rede (Ethernet, WiFi, Bluetooth)
    /// Conectam o sistema a redes externas
    Network,

    /// Dispositivos de comunicação serial (UART, COM, USB-Serial)
    /// Usados para debug e dispositivos industriais
    Serial,

    /// Dispositivos de áudio (Placas de som, CODECs)
    /// Produzem e capturam som
    Audio,

    /// Controladores de barramento (Hubs USB, Bridges PCI)
    /// Descobrem e gerenciam outros dispositivos
    Bus,

    /// Fontes de tempo (PIT, HPET, TSC, RTC)
    /// Críticos para o scheduler e timekeeping
    Timer,

    /// Controladores de sistema (DMA, Interrupt Controllers, IOMMU)
    /// Infraestrutura crítica do hardware
    Controller,

    /// Dispositivos de segurança (TPM, Hardware RNG)
    /// Criptografia e geração de números aleatórios
    Security,

    /// Dispositivos que não se encaixam em outras categorias
    /// Sensores, atuadores, GPIOs, etc
    Generic,

    /// Tipo desconhecido (dispositivo ainda não identificado)
    Unknown,

    /// Dispositivos de infraestrutura de sistema (Relógios, Controladores de interrupção, etc)
    System,
}

impl DeviceType {
    /// Retorna nome legível do tipo de dispositivo.
    ///
    /// Útil para logs e interface de usuário.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Storage => "Storage",
            Self::Input => "Input",
            Self::Display => "Display",
            Self::Network => "Network",
            Self::Serial => "Serial",
            Self::Audio => "Audio",
            Self::Bus => "Bus",
            Self::Timer => "Timer",
            Self::Controller => "Controller",
            Self::Security => "Security",
            Self::Generic => "Generic",
            Self::Unknown => "Unknown",
            Self::System => "System",
        }
    }
}

// =============================================================================
// ERROS DE DRIVER
// =============================================================================

/// Erros que podem ocorrer durante operações de driver.
///
/// Cada erro tem semântica específica que o RecoveryManager usa para
/// decidir a estratégia de recuperação apropriada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// Hardware não é suportado por este driver.
    /// Erro normal durante probe - driver simplesmente não é compatível.
    /// Recovery: Tentar próximo driver disponível.
    NotSupported,

    /// Falha crítica durante inicialização dos registradores.
    /// Geralmente indica problema de hardware ou firmware.
    /// Recovery: Reset de hardware, depois fallback.
    InitFailed,

    /// Erro de comunicação no barramento (Timeout, NACK, CRC).
    /// Pode ser problema transiente (mau contato) ou permanente.
    /// Recovery: Retry, depois reset de bus.
    BusError,

    /// O hardware reportou erro interno via status register.
    /// Indica problema no próprio dispositivo.
    /// Recovery: Reset de dispositivo, depois isolamento.
    DeviceError,

    /// O hardware parou de responder dentro do tempo esperado.
    /// Pode indicar travamento ou desconexão.
    /// Recovery: Reset, depois fallback.
    Timeout,

    /// Parâmetros de configuração inválidos foram fornecidos.
    /// Geralmente bug no driver ou incompatibilidade.
    /// Recovery: Usar configuração padrão.
    InvalidParameter,

    /// Falha ao alocar memória DMA ou buffers necessários.
    /// Sistema pode estar com memória baixa.
    /// Recovery: Liberar buffers, tentar novamente com menos memória.
    NoMemory,

    /// Acesso negado (BIOS lock, firmware protection).
    /// Hardware está protegido contra modificações.
    /// Recovery: Fallback para driver genérico.
    AccessDenied,

    /// O recurso solicitado (IRQ, porta, MMIO) já está em uso.
    /// Conflito com outro driver.
    /// Recovery: Tentar recursos alternativos.
    ResourceConflict,

    /// Operação foi cancelada (por usuário ou sistema).
    /// Não é erro real, apenas interrupção.
    /// Recovery: Nenhuma necessária.
    Cancelled,

    /// Driver está sendo descarregado (hot-reload em andamento).
    /// Operações devem ser adiadas.
    /// Recovery: Aguardar reload completar.
    Unloading,

    /// Falha física detectada no hardware.
    HardwareFault,

    /// Erro de hardware genérico.
    HardwareError,
}

impl DriverError {
    /// Retorna se o erro é considerado crítico.
    ///
    /// Erros críticos contam mais no score de saúde do dispositivo
    /// e podem levar a isolamento mais rápido.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::InitFailed | Self::AccessDenied | Self::NoMemory | Self::DeviceError
        )
    }

    /// Retorna se o erro pode ser transiente (mau contato, etc).
    ///
    /// Erros transientes merecem mais tentativas de retry antes
    /// de escalar para ações mais drásticas.
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::BusError | Self::Timeout)
    }

    /// Retorna nome legível do erro.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotSupported => "NotSupported",
            Self::InitFailed => "InitFailed",
            Self::BusError => "BusError",
            Self::DeviceError => "DeviceError",
            Self::Timeout => "Timeout",
            Self::InvalidParameter => "InvalidParameter",
            Self::NoMemory => "NoMemory",
            Self::AccessDenied => "AccessDenied",
            Self::ResourceConflict => "ResourceConflict",
            Self::Cancelled => "Cancelled",
            Self::Unloading => "Unloading",
            Self::HardwareFault => "HardwareFault",
            Self::HardwareError => "HardwareError",
        }
    }
}

// =============================================================================
// TRAIT PRINCIPAL: DRIVER
// =============================================================================

/// Trait fundamental para todos os drivers do RedstoneOS.
///
/// Implementar esta trait torna o driver um "cidadão de primeira classe"
/// no RDS, permitindo que ele seja:
/// - Registrado no sistema
/// - Pareado com dispositivos compatíveis
/// - Gerenciado pelo RecoveryManager
/// - Sujeito a hot-reload
///
/// ## Métodos Obrigatórios:
/// - `name()`: Identificação única
/// - `device_type()`: Classificação funcional
/// - `probe()`: Tentar assumir controle de dispositivo
/// - `remove()`: Liberar recursos e parar hardware
///
/// ## Métodos Opcionais:
/// - `suspend()`, `resume()`: Gerenciamento de energia
/// - `shutdown()`: Desligamento do sistema
/// - `abi_version()`: Versão da ABI para compatibilidade
/// - `capabilities()`: Flags de capacidades especiais
pub trait Driver: Send + Sync {
    // =========================================================================
    // IDENTIFICAÇÃO (OBRIGATÓRIO)
    // =========================================================================

    /// Retorna o nome identificador único do driver.
    ///
    /// Este nome é usado para:
    /// - Logs e debugging
    /// - Referência em comandos de hot-reload
    /// - Exibição para o usuário
    ///
    /// ## Convenção de Nomenclatura:
    /// Use formato `categoria-nome-driver`, exemplo:
    /// - "virtio-blk-driver"
    /// - "intel-e1000-network"
    /// - "ps2-keyboard-input"
    fn name(&self) -> &'static str;

    /// Retorna o tipo funcional primário do dispositivo.
    ///
    /// Determina em qual classe funcional o dispositivo será registrado.
    /// Um driver deve ter um tipo primário claro, mesmo que suporte
    /// funcionalidades secundárias.
    fn device_type(&self) -> DeviceType;

    // =========================================================================
    // CICLO DE VIDA (OBRIGATÓRIO)
    // =========================================================================

    /// Tenta assumir controle de um dispositivo.
    ///
    /// Chamado pelo DriverManager quando um novo dispositivo é descoberto
    /// ou quando um driver é registrado e existem dispositivos órfãos.
    ///
    /// ## Responsabilidades do probe():
    /// 1. Verificar IDs de hardware (Vendor/Device) - retornar NotSupported se não for compatível
    /// 2. Solicitar recursos via `base::resource::request_*()` - NUNCA alocar direto!
    /// 3. Inicializar registradores base do hardware
    /// 4. Opcionalmente, registrar em classes funcionais
    ///
    /// ## Retorno:
    /// - `Ok(())`: Driver assumiu controle com sucesso
    /// - `Err(NotSupported)`: Driver não é compatível com este hardware
    /// - `Err(outro)`: Erro durante inicialização
    ///
    /// ## IMPORTANTE:
    /// - Não fazer alocações pesadas aqui (use lazy init se necessário)
    /// - Não bloquear por longos períodos (use async se necessário)
    /// - Sempre verificar IDs antes de assumir controle
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;

    /// Libera controle do dispositivo e limpa recursos.
    ///
    /// Chamado quando:
    /// - O dispositivo é removido fisicamente (hotplug)
    /// - O driver está sendo descarregado (hot-reload)
    /// - O sistema está desligando
    /// - O Recovery está tentando rebind
    ///
    /// ## Responsabilidades do remove():
    /// 1. Parar toda atividade de DMA imediatamente
    /// 2. Desabilitar interrupções do dispositivo
    /// 3. Liberar recursos (automático se usou `base::resource`)
    /// 4. Limpar estruturas internas
    ///
    /// ## IMPORTANTE:
    /// - Garantir que o hardware pare de gerar IRQs
    /// - Garantir que DMA pare antes de liberar buffers
    /// - Não falhar se possível (melhor esforço)
    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }

    // =========================================================================
    // GERENCIAMENTO DE ENERGIA (OPCIONAL)
    // =========================================================================

    /// Coloca o dispositivo em modo de baixo consumo.
    ///
    /// Chamado durante suspensão do sistema ou quando o dispositivo
    /// não está sendo usado (runtime PM).
    ///
    /// ## Tarefas típicas:
    /// - Salvar estado de registradores
    /// - Desabilitar clocks
    /// - Colocar hardware em D3 (off)
    ///
    /// ## Default:
    /// Implementação padrão não faz nada (Ok).
    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Override se o driver suporta PM
        Ok(())
    }

    /// Restaura o dispositivo do modo de baixo consumo.
    ///
    /// Chamado ao acordar do suspend ou quando dispositivo volta a ser usado.
    ///
    /// ## Tarefas típicas:
    /// - Restaurar estado de registradores
    /// - Reconfigurar hardware
    /// - Retomar operações pendentes
    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Override se o driver suporta PM
        Ok(())
    }

    /// Prepara o hardware para desligamento do sistema.
    ///
    /// Diferente de `remove()`, aqui o objetivo não é liberar recursos
    /// para reutilização, mas sim colocar o hardware em estado seguro
    /// para power-off.
    ///
    /// ## Tarefas típicas:
    /// - Flush de caches para disco
    /// - Desligar motores de HDD
    /// - Neutralizar saídas (ex: parar de tocar áudio)
    fn shutdown(&self, _dev: &mut Device) {
        // Override se ações especiais são necessárias
    }

    // =========================================================================
    // COMPATIBILIDADE E CAPABILITIES (OPCIONAL)
    // =========================================================================

    /// Retorna a versão da ABI do driver.
    ///
    /// Usado para verificar compatibilidade durante hot-reload de módulos externos.
    /// Formato: 0xMM_mm_pp_00 (Major.Minor.Patch.Reserved)
    ///
    /// ## Default:
    /// Retorna versão atual do RDS (1.0.0)
    fn abi_version(&self) -> u32 {
        super::RDS_ABI_VERSION
    }

    /// Retorna flags de capacidades especiais do driver.
    ///
    /// Bitmask que indica funcionalidades opcionais suportadas.
    /// Definir constantes CAP_* conforme necessário.
    ///
    /// ## Default:
    /// Retorna 0 (sem capacidades especiais)
    fn capabilities(&self) -> u64 {
        0
    }

    /// Retorna lista de IDs de dispositivos suportados.
    ///
    /// Permite que o DriverManager faça matching rápido sem precisar
    /// chamar probe() em drivers incompatíveis.
    ///
    /// ## STUB - Futuro
    /// Será implementado quando tivermos Device ID Tables.
    fn supported_ids(&self) -> &'static [(u16, u16)] {
        // (VendorID, DeviceID) - vazio significa "probe necessário"
        &[]
    }
}

// =============================================================================
// CONSTANTES DE CAPABILITIES
// =============================================================================

/// Driver suporta DMA (pode usar o DmaPool)
pub const CAP_DMA: u64 = 1 << 0;

/// Driver suporta gerenciamento de energia
pub const CAP_POWER_MANAGEMENT: u64 = 1 << 1;

/// Driver suporta hot-reload
pub const CAP_HOT_RELOAD: u64 = 1 << 2;

/// Driver suporta operações assíncronas
pub const CAP_ASYNC: u64 = 1 << 3;

/// Driver é crítico para o sistema (não pode falhar silenciosamente)
pub const CAP_CRITICAL: u64 = 1 << 4;

/// Driver requer firmware externo
pub const CAP_NEEDS_FIRMWARE: u64 = 1 << 5;
