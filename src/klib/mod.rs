//! Kernel Library
//!
//! Substitui partes da std e fornece utilitários básicos.

pub mod align;
pub mod bitmap;
pub mod mem_funcs;
pub mod test_framework;
#[macro_use]
pub mod bitflags;

pub mod hash;
pub mod list;
pub mod string;
pub mod tree;

pub use align::{align_down, align_up, is_aligned};
pub use bitmap::Bitmap;
