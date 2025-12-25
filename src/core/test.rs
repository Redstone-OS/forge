//! Testes do Core/Kernel Main
//!
//! # Por que testar?
//! O core orquestra a subida do sistema. Falhas aqui resultam em um kernel "mudo" ou que trava antes de
//! chegar no primeiro processo. O parser ELF é crítico porque é ele quem transforma binários em processos.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_boot_info_validation`:
//!    - O que: Simular BootInfo corrompido ou com versão incompatível.
//!    - Por que: Garante que o kernel se protege contra bootloaders desalinhados ou dados lixo da memória.
//!
//! 2. `test_elf_parser`:
//!    - O que: Passar headers ELF propositalmente inválidos para o loader.
//!    - Por que: Evita que o kernel tente executar código de arquivos corrompidos, prevenindo crashes aleatórios.
//!
//! 3. `test_entry_point_consistency`:
//!    - O que: Verificar se o entry point extraído do ELF aponta para uma região de memória executável.
//!    - Por que: Garante que o salto para o processo `init` não cairá em uma região de dados ou memória não mapeada.
//!
//! 4. `test_handoff_structures`:
//!    - O que: Verificar a integridade das estruturas de handoff após a inicialização.
//!    - Por que: Assegura que informações do bootloader (como o framebuffer) persistem corretamente no estado interno do kernel.
