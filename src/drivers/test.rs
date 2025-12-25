//! Testes de Drivers e Hardware I/O
//!
//! # Por que testar?
//! Drivers mal configurados podem causar interrupções espúrias (falsas) que travam o sistema ou corrompem
//! o tempo do sistema. Sem o Timer (PIT), o Scheduler morre. Sem a Serial, perdemos a telemetria do kernel.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_pit_heartbeat`:
//!    - O que: Medir o tempo entre interrupções do timer (usando RDTSC como base comparativa).
//!    - Por que: Garante que a frequência de 100Hz (10ms) está correta para o agendamento de tarefas.
//!
//! 2. `test_pic_masking`:
//!    - O que: Mascarar uma interrupção (ex: teclado) e verificar se ela deixa de ser processada.
//!    - Por que: Valida o controle do kernel sobre o fluxo de hardware, evitando "tempestades de interrupção".
//!
//! 3. `test_serial_loopback`:
//!    - O que: Escrever na porta serial e verificar se os buffers internos não transbordam.
//!    - Por que: O log é nossa principal ferramenta de debug; ele precisa ser confiável e performático.
//!
//! 4. `test_framebuffer_access`:
//!    - O que: Tentar escrever padrões simples no início e no fim da memória de vídeo.
//!    - Por que: Valida se o mapeamento do Framebuffer passado pelo bootloader está acessível e correto.
