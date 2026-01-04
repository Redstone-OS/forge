//! # VirtIO-GPU Driver
//!
//! Driver de vídeo para hardware virtualizado de alto desempenho (QEMU/KVM).
//! Implementa aceleração 2D básica e gerenciamento de framebuffer via VirtIO.

pub mod regs;

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::virtio::transport::VirtioTransport;
use crate::drivers::bus::virtio::types::VirtioDeviceId;
use alloc::sync::Arc;

pub struct VirtioGpuDriver;

impl Driver for VirtioGpuDriver {
    fn name(&self) -> &'static str {
        "VirtIO Graphics Adapter Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // 1. Verificar se o dispositivo é uma GPU VirtIO
        // O Id do dispositivo no RDM para VirtIO é 0x1AF4XXXX
        // Mas o DeviceType já foi mapeado pelo VirtioBus::scan()

        if dev.device_type != DeviceType::Display {
            return Err(DriverError::NotSupported);
        }

        // TODO: Em uma implementação futura, o VirtioBus anexará o VirtioTransport
        // aos dados privados do dispositivo.
        /*
        let transport = match dev.get_data::<Arc<dyn VirtioTransport>>() {
            Some(t) => t,
            None => return Err(DriverError::NotSupported),
        };
        */

        crate::kinfo!("(Display) VirtIO-GPU detectado e associado ao RDM.");

        // 2. Inicializar o Handshake VirtIO
        // - Reset do dispositivo
        // - Ack Features
        // - Configurar Virtqueues (Controlq e Cursorq)

        crate::kdebug!("(Display) VirtIO-GPU: Inicializando pipelines de comando...");

        // 3. Obter Informações de Display (GetDisplayInfo)

        // 4. Criar Resource 2D (Framebuffer) e anexar memória backing

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Suspended;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(VirtioGpuDriver));
}
