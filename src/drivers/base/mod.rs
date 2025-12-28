//! Modelo de Drivers Base

pub mod bus;
pub mod class;
pub mod device;
pub mod driver;

pub use device::Device;
pub use driver::Driver;
pub use driver::DriverError;
pub use driver::DeviceType;

/// Inicializa subsistema de drivers base
pub fn init() {
    // TODO: Inicializar driver manager/registry
}
