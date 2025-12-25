//! Testes de Chamadas de Sistema (Syscalls)
//!
//! # Por que testar?
//! As syscalls são o portal entre o mundo inseguro (usuário) e o mundo seguro (kernel).
//! Validar os argumentos passados é a linha de frente contra ataques que tentam "enganar" o kernel.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_syscall_dispatch`:
//!    - O que: Chamar uma syscall inexistente e verificar se o kernel retorna o erro correto (-1/ENOSYS).
//!    - Por que: Garante que a tabela de saltos das syscalls está protegida contra índices fora dos limites.
//!
//! 2. `test_invalid_pointer_argument`:
//!    - O que: Passar um ponteiro nulo ou para memória do kernel numa syscall de escrita (ex: `write`).
//!    - Por que: Verifica se o kernel valida os endereços antes de tentar acessá-los, evitando Page Faults no Ring 0.
//!
//! 3. `test_argument_count_limit`:
//!    - O que: Tentar passar mais argumentos do que a arquitetura permite nos registradores.
//!    - Por que: Garante que o kernel ignora ou lida corretamente com dados extras que poderiam causar transbordo de stack.
//!
//! 4. `test_syscall_performance_impact`:
//!    - O que: Medir o overhead de uma syscall nula (ex: `get_pid`).
//!    - Por que: As syscalls devem ser o mais rápidas possível para garantir a performance geral do sistema.
