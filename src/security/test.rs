//! Testes de Segurança e Controle de Acesso (Capabilities)
//!
//! # Por que testar?
//! O Redstone OS usa um modelo baseado em Capabilities (habilidades). Se um processo consegue forjar
//! uma permissão que não deveria ter, todo o isolamento do sistema é comprometido.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_capability_delegation`:
//!    - O que: Tentar passar uma permissão de Leitura de um processo A para um B e verificar se B consegue ler o arquivo.
//!    - Por que: Valida o mecanismo de propagação de direitos entre processos.
//!
//! 2. `test_access_denied_enforcement`:
//!    - O que: Tentar realizar uma operação proibida (ex: escrever em arquivo protegido) e verificar se o kernel bloqueia.
//!    - Por que: É a garantia de que as barreiras de segurança estão ativas e funcionando.
//!
//! 3. `test_capability_revocation`:
//!    - O que: Retirar um direito de uma tarefa e verificar se a próxima tentativa de uso falha imediatamente.
//!    - Por que: Garante que o sistema consegue reagir a mudanças de privilégio em tempo real.
//!
//! 4. `test_resource_isolation`:
//!    - O que: Tentar acessar portas de IPC de outro processo sem ter o handle correspondente.
//!    - Por que: Mantém o isolamento entre subsistemas, impedindo que um driver com bug afete outros.
