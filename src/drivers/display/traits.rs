//! # Display Device Traits and Types
//!
//! Define as interfaces e tipos fundamentais para dispositivos de display.
//!
//! ## Arquitetura RDS:
//! Todos os drivers de display implementam `DisplayDevice` e são registrados
//! no subsistema central.
//!
//! ## Tipos de Dispositivos:
//! - **GOP/VBE**: Framebuffer do bootloader
//! - **Bochs BGA**: QEMU/Bochs virtual display
//! - **VirtIO-GPU**: Paravirtualizado com aceleração 2D
//! - **Intel/AMD/NVIDIA**: GPUs reais

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// FORMATOS DE PIXEL
// =============================================================================

/// Formato de pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelFormat {
    /// 32-bit ARGB (Alpha, Red, Green, Blue).
    #[default]
    Argb8888,
    /// 32-bit BGRA (Blue, Green, Red, Alpha).
    Bgra8888,
    /// 32-bit XRGB (ignored Alpha).
    Xrgb8888,
    /// 24-bit RGB (sem alpha).
    Rgb888,
    /// 16-bit RGB (5-6-5).
    Rgb565,
    /// 8-bit palletized.
    Indexed8,
}

impl PixelFormat {
    /// Bytes por pixel.
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Argb8888 | Self::Bgra8888 | Self::Xrgb8888 => 4,
            Self::Rgb888 => 3,
            Self::Rgb565 => 2,
            Self::Indexed8 => 1,
        }
    }

    /// Bits por pixel.
    pub const fn bits_per_pixel(&self) -> u8 {
        (self.bytes_per_pixel() * 8) as u8
    }
}

// =============================================================================
// MODOS DE VÍDEO
// =============================================================================

/// Modo de vídeo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMode {
    /// Largura em pixels.
    pub width: u32,
    /// Altura em pixels.
    pub height: u32,
    /// Formato de pixel.
    pub format: PixelFormat,
    /// Taxa de refresh em mHz (ex: 60000 = 60Hz).
    pub refresh_rate_mhz: u32,
}

impl VideoMode {
    /// Modo padrão 1024x768@60Hz.
    pub const DEFAULT: Self = Self {
        width: 1024,
        height: 768,
        format: PixelFormat::Argb8888,
        refresh_rate_mhz: 60000,
    };

    /// Stride em bytes (assumindo sem padding).
    pub const fn stride(&self) -> u32 {
        self.width * self.format.bytes_per_pixel() as u32
    }

    /// Tamanho total do framebuffer em bytes.
    pub const fn framebuffer_size(&self) -> usize {
        (self.stride() * self.height) as usize
    }
}

impl Default for VideoMode {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// =============================================================================
// ERROS DE DISPLAY
// =============================================================================

/// Erros de operações de display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayError {
    /// Display não inicializado.
    NotInitialized,
    /// Modo não suportado.
    UnsupportedMode,
    /// Resolução inválida.
    InvalidResolution,
    /// Formato de pixel não suportado.
    UnsupportedFormat,
    /// Buffer inválido.
    InvalidBuffer,
    /// Page flip pendente.
    FlipPending,
    /// Erro de hardware.
    HardwareError,
    /// Sem memória de vídeo.
    OutOfMemory,
    /// Timeout.
    Timeout,
    /// Operação não suportada.
    NotSupported,
}

impl DisplayError {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotInitialized => "Display Not Initialized",
            Self::UnsupportedMode => "Unsupported Video Mode",
            Self::InvalidResolution => "Invalid Resolution",
            Self::UnsupportedFormat => "Unsupported Pixel Format",
            Self::InvalidBuffer => "Invalid Buffer",
            Self::FlipPending => "Page Flip Pending",
            Self::HardwareError => "Hardware Error",
            Self::OutOfMemory => "Out of Video Memory",
            Self::Timeout => "Timeout",
            Self::NotSupported => "Operation Not Supported",
        }
    }
}

// =============================================================================
// CAPACIDADES DO DISPLAY
// =============================================================================

/// Capacidades de um dispositivo de display.
#[derive(Debug, Clone, Default)]
pub struct DisplayCapabilities {
    /// Modos suportados.
    pub modes: Vec<VideoMode>,
    /// Largura máxima.
    pub max_width: u32,
    /// Altura máxima.
    pub max_height: u32,
    /// Suporta double buffering.
    pub double_buffer: bool,
    /// Suporta page flip (VSync).
    pub page_flip: bool,
    /// Suporta hardware cursor.
    pub hw_cursor: bool,
    /// Suporta aceleração 2D.
    pub accel_2d: bool,
    /// Suporta aceleração 3D.
    pub accel_3d: bool,
    /// Quantidade de VRAM em bytes.
    pub vram_size: usize,
}

// =============================================================================
// INFORMAÇÕES DO DISPLAY
// =============================================================================

/// Informações de identificação do display.
#[derive(Debug, Clone, Default)]
pub struct DisplayInfo {
    /// Nome do dispositivo.
    pub name: String,
    /// Modelo/descrição.
    pub model: String,
    /// Modo atual.
    pub current_mode: VideoMode,
    /// Endereço do framebuffer.
    pub framebuffer_addr: u64,
    /// Stride atual em bytes.
    pub stride: u32,
}

// =============================================================================
// ESTATÍSTICAS
// =============================================================================

/// Estatísticas de um display.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayStats {
    /// Frames renderizados.
    pub frames: u64,
    /// Page flips executados.
    pub flips: u64,
    /// Bytes transferidos para VRAM.
    pub bytes_transferred: u64,
    /// Número de underruns (frame perdido).
    pub underruns: u32,
}

// =============================================================================
// TRAIT PRINCIPAL: DISPLAY DEVICE
// =============================================================================

/// Interface base para dispositivos de display.
pub trait DisplayDevice: Send + Sync {
    // -------------------------------------------------------------------------
    // Identificação
    // -------------------------------------------------------------------------

    /// Nome do dispositivo.
    fn name(&self) -> &str;

    /// Informações detalhadas.
    fn info(&self) -> DisplayInfo;

    // -------------------------------------------------------------------------
    // Modos de Vídeo
    // -------------------------------------------------------------------------

    /// Retorna modo atual.
    fn current_mode(&self) -> VideoMode;

    /// Lista modos suportados.
    fn supported_modes(&self) -> Vec<VideoMode>;

    /// Define modo de vídeo.
    fn set_mode(&self, mode: VideoMode) -> Result<(), DisplayError>;

    /// Capacidades do dispositivo.
    fn capabilities(&self) -> DisplayCapabilities;

    // -------------------------------------------------------------------------
    // Framebuffer
    // -------------------------------------------------------------------------

    /// Retorna ponteiro para o framebuffer.
    fn framebuffer(&self) -> *mut u8;

    /// Retorna tamanho do framebuffer.
    fn framebuffer_size(&self) -> usize;

    /// Stride do framebuffer em bytes.
    fn stride(&self) -> u32;

    // -------------------------------------------------------------------------
    // Operações
    // -------------------------------------------------------------------------

    /// Limpa o display com uma cor.
    fn clear(&self, color: u32);

    /// Executa page flip.
    fn flip(&self) -> Result<(), DisplayError> {
        Ok(())
    }

    /// Espera VSync.
    fn wait_vsync(&self) {}

    // -------------------------------------------------------------------------
    // Estado
    // -------------------------------------------------------------------------

    /// Habilita o display.
    fn enable(&self) -> Result<(), DisplayError>;

    /// Desabilita o display.
    fn disable(&self);

    /// Verifica se está habilitado.
    fn is_enabled(&self) -> bool;

    /// Estatísticas.
    fn get_stats(&self) -> DisplayStats {
        DisplayStats::default()
    }
}

pub type DisplayDeviceRef = Arc<dyn DisplayDevice>;

// =============================================================================
// CONNECTOR TYPES
// =============================================================================

/// Tipo de conector físico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    Unknown,
    Vga,
    DviI,
    DviD,
    DviA,
    Hdmi,
    DisplayPort,
    Edp,
    Lvds,
    Virtual,
}

/// Estado do conector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorStatus {
    Connected,
    Disconnected,
    Unknown,
}
