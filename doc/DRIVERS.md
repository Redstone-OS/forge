# Drivers e Hardware

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Console e Serial](#console-e-serial)
- [Temporizador (Timer)](#temporizador-timer)
- [Controlador de Interrupções (PIC)](#controlador-de-interrupções-pic)
- [Vídeo](#vídeo)

---

## Visão Geral

O Forge implementa drivers de dispositivo no modo kernel (Ring 0) para garantir performance e acesso direto ao hardware. Os drivers estão localizados em `src/drivers/`.

### Estrutura
-   **`serial.rs`**: Comunicação serial (UART 16550) para logs de debug.
-   **`console.rs`**: Abstração de saída de texto (escreve no serial e/ou vídeo).
-   **`pic.rs`**: Programmable Interrupt Controller (8259 PIC), usado para mapear IRQs de hardware.
-   **`timer.rs`**: Programmable Interval Timer (PIT) ou APIC Timer para scheduling.
-   **`video/`**: Suporte a Framebuffer gráfico (GOP) herdado do UEFI.

---

## Console e Serial

O **Serial Port (COM1)** é o principal canal de debug do kernel, pois é simples e confiável.
-   **Porta IO**: `0x3F8`
-   **Baud Rate**: 115200 (configurado pelo bootloader ou driver)

O **Console** combina a saída serial com o framebuffer de vídeo, permitindo `printk!` que aparece tanto no QEMU monitor (stdio) quanto na tela da VM.

---

## Temporizador (Timer)

O kernel precisa de uma fonte de tempo periódica para implementar multitarefa preemptiva.

### PIT (Programmable Interval Timer)
-   Configurado para disparar IRQ 0 a uma frequência fixa (ex: 100Hz ou 1000Hz).
-   A cada "tick", o scheduler é invocado para decidir se deve trocar de tarefa.

---

## Controlador de Interrupções (PIC)

O **8259 PIC** é um controlador legado, mas ainda usado para bootstrap ou em sistemas simples. O Forge remapeia as interrupções do PIC para não conflitarem com as exceções da CPU (0-31).

-   **Master PIC**: Mapeado para vetor 32 (Offset 0x20).
-   **Slave PIC**: Mapeado para vetor 40 (Offset 0x28).

Isso significa que a IRQ 0 (Timer) chega na CPU como Interrupção 32.

---

## Vídeo

O suporte a vídeo é baseado em **Framebuffer Linear**.
-   O endereço do framebuffer, largura, altura e pitch são passados pelo `Ignite Bootloader`.
-   O kernel não muda a resolução (isso é feito pelo bootloader).
-   O driver de vídeo apenas desenha pixels na memória mapeada.
