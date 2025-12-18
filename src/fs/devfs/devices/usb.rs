//! /dev/usb/* - Dispositivos USB
//!
//! TODO(prioridade=baixa, versão=v2.0): Implementar USB
//!
//! # Arquitetura Userspace
//! - **Kernel:** xHCI/EHCI/UHCI driver (DMA, interrupções)
//! - **Userspace:** Pilha USB, drivers de dispositivos
//!
//! # Dispositivos
//! - /dev/usb/hiddev*: HID devices
//! - /dev/usb/lp*: Impressoras USB
//! - /dev/bus/usb/: usbfs (acesso direto)
//!
//! # Vantagens Userspace
//! - USB é complexo e bugado
//! - Crash não mata o kernel
//! - Fácil atualização de drivers
//!
//! # Implementação Sugerida
//! - Kernel: xHCI driver básico
//! - Userspace: libusb, drivers específicos

// TODO: Implementar UsbDevice
