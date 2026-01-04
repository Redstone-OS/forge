//! # Ethernet Driver Manager
//!
//! Orquestra os drivers específicos de hardware ethernet.

pub mod intel;
pub mod realtek;

use crate::drivers::base::driver::Driver;
use alloc::sync::Arc;

pub fn init() {
    // Registra os drivers no RDM
    crate::drivers::base::register_driver(Arc::new(intel::IntelNetDriver::new()) as Arc<dyn Driver>);
    crate::drivers::base::register_driver(Arc::new(realtek::RealtekNetDriver) as Arc<dyn Driver>);
}
