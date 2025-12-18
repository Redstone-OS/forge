//! Drivers do Kernel
//!
//! Drivers organizados por barramento.
//!
//! # Organização
//! - PCI: Dispositivos PCI
//! - USB: Dispositivos USB (xHCI)
//! - Platform: Dispositivos de plataforma
//! - Legacy: Dispositivos legados (PS/2, Serial, VGA)
//!
//! # Drivers Essenciais para v1.0
//! - Serial (UART 16550)
//! - PS/2 (Teclado e mouse)
//! - VGA/GOP (Display básico)
//! - AHCI (SATA)
//! - E1000 (Rede Intel)
//! - RTL8139 (Rede Realtek)
//! - xHCI (USB 3.0)
//!
//! # Modelo
//! Híbrido: Drivers críticos no kernel, resto em userspace
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Migrar drivers existentes
//! - TODO(prioridade=alta, versão=v1.0): Implementar drivers essenciais
//! - TODO(prioridade=média, versão=v2.0): Mover drivers não-críticos para userspace

pub mod input_buffer;
pub mod keyboard;
pub mod legacy;
pub mod pci;
pub mod pic;
pub mod platform;
pub mod timer;
pub mod usb;
pub mod video;
