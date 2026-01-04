//! # Intel High Definition Audio (HDA)
//!
//! Driver principal para áudio em PCs modernos (Intel HDA / Azalia).

pub mod codecs;
pub mod intel; // Driver específico da Intel // Parser de codecs (Realtek, etc)

pub use intel::IntelHdaDriver;
