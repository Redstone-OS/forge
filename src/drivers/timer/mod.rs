//! Timer Drivers

pub mod hpet;
pub mod pit;
pub mod tsc;

pub use pit::init as init_pit;
