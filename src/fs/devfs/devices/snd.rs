//! /dev/snd/* - Dispositivos de áudio
//!
//! TODO(prioridade=baixa, versão=v2.0): Implementar áudio
//!
//! # Arquitetura Userspace
//! - **Kernel:** Apenas DMA e interrupções
//! - **Userspace:** Mixing, processamento, efeitos
//!
//! # Dispositivos
//! - /dev/snd/pcmC0D0p: Playback PCM
//! - /dev/snd/pcmC0D0c: Capture PCM
//! - /dev/snd/controlC0: Controle de mixer
//! - /dev/snd/timer: Timer de áudio
//!
//! # Implementação Sugerida
//! - Compatível com ALSA
//! - Drivers: AC97, HDA, USB Audio
//! - Userspace: PulseAudio/PipeWire

// TODO: Implementar SndDevice
