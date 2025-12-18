//! Devices - Implementações de dispositivos específicos

// Dispositivos essenciais (implementados)
pub mod console;
pub mod mem;
pub mod null;
pub mod rtc;
pub mod tty;
pub mod zero;

// Dispositivos opcionais (TODOs)
pub mod fb;
pub mod input;
pub mod net;
pub mod random;
pub mod snd;
pub mod usb;

// Re-exports para facilitar uso
pub use console::ConsoleDevice;
pub use mem::{KmemDevice, MemDevice};
pub use null::NullDevice;
pub use rtc::RtcDevice;
pub use tty::{TtyDevice, TtyS0Device};
pub use zero::ZeroDevice;
