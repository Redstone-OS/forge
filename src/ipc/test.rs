//! Testes de Comunicação entre Processos (IPC)
//!
//! # Por que testar?
//! Redstone OS visa uma arquitetura micro-modular. Se o IPC falhar, os serviços do sistema (drivers de rede, disk)
//! não conseguirão responder ao usuário. É a "espinha dorsal" de um sistema operacional moderno.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_port_creation_leak`:
//!    - O que: Abrir e destruir milhares de portas de comunicação em loop.
//!    - Por que: Garante que o kernel limpa corretamente os metadados das portas, evitando OOM (Out of Memory) lógico.
//!
//! 2. `test_message_ordering`:
//!    - O que: Enviar 100 mensagens em ordem e verificar se chegam na mesma sequência na outra ponta.
//!    - Por que: A previsibilidade no protocolo é essencial para o correto funcionamento dos drivers e servidores.
//!
//! 3. `test_blocking_receive`:
//!    - O que: Colocar uma tarefa para dormir esperando uma mensagem e acordá-la enviando o dado.
//!    - Por que: Valida a integração entre IPC e Scheduler, garantindo que o CPU não seja desperdiçado em loops de espera.
//!
//! 4. `test_capacity_overflow`:
//!    - O que: Enviar mensagens para uma porta cujo buffer está cheio e verificar se o kernel retorna erro corretamente.
//!    - Por que: Evita que um processo mal-intencionado trave o kernel enviando spam de mensagens.
