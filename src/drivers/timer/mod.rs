//! Timer Drivers

pub mod pit;
pub mod hpet;
pub mod tsc;

pub use pit::init as init_pit;
