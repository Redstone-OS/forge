//! /dev/random e /dev/urandom - Geradores de números aleatórios
//!
//! TODO(prioridade=média, versão=v1.0): Implementar gerador de números aleatórios
//!
//! # Implementação Sugerida
//! - Usar RDRAND/RDSEED (x86_64)
//! - Pool de entropia
//! - /dev/random: blocking (espera entropia)
//! - /dev/urandom: non-blocking (CSPRNG)

// TODO: Implementar RandomDevice e UrandomDevice
