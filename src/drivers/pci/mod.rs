//! Drivers PCI
//!
//! Drivers para dispositivos PCI.
//!
//! # Drivers
//! - AHCI (SATA)
//! - E1000 (Rede Intel)
//! - RTL8139 (Rede Realtek)
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar AHCI
//! - TODO(prioridade=alta, versão=v1.0): Implementar E1000
//! - TODO(prioridade=alta, versão=v1.0): Implementar RTL8139

pub mod ahci;
pub mod e1000;
pub mod rtl8139;
