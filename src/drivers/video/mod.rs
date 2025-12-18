//! Video Driver Module
//!
//! Provides video output capabilities for the kernel.
//! Includes framebuffer driver, font rendering, and text console.

pub mod console;
pub mod font;
pub mod framebuffer;

pub use console::Console;
pub use framebuffer::{Color, Framebuffer};
