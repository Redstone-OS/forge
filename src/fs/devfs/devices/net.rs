//! /dev/net/* - Dispositivos de rede
//!
//! TODO(prioridade=média, versão=v1.0): Implementar rede
//!
//! # Arquitetura Híbrida
//! - **Kernel:** NIC driver (e1000, rtl8139), interrupções, DMA
//! - **Userspace:** TCP/IP stack (opcional, como Redox)
//!
//! # Dispositivos
//! - /dev/net/tun: TUN/TAP (túneis)
//! - /dev/net/tap: TAP (ethernet virtual)
//!
//! # Opções de Implementação
//!
//! ## Opção A: Kernel TCP/IP (Linux-style)
//! - Vantagem: Performance máxima
//! - Desvantagem: Complexo, bugs no kernel
//!
//! ## Opção B: Userspace TCP/IP (Redox-style)
//! - Vantagem: Seguro, modular
//! - Desvantagem: Latência ligeiramente maior
//! - Usar: smoltcp crate
//!
//! # Recomendação
//! - Kernel: NIC driver + raw sockets
//! - Userspace: TCP/IP stack (smoltcp)

// TODO: Implementar NetDevice, TunDevice, TapDevice
