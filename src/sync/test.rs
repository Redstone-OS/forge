//! Testes de Sincronização (Spinlocks, Mutexes, Atômicos)
//!
//! # Por que testar?
//! Em um sistema multicore, a sincronização é o que evita o caos. Race Conditions (condições de corrida)
//! são erros difíceis de reproduzir; testes de estresse aqui são vitais para a estabilidade.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_spinlock_contention`:
//!    - O que: Múltiplas "kernel threads" tentando incrementar um contador protegido por um Spinlock.
//!    - Por que: Valida a exclusão mútua básica e garante que o contador final seja coerente.
//!
//! 2. `test_mutex_blocking`:
//!    - O que: Verificar se uma tarefa que tenta pegar um Mutex já ocupado é corretamente colocada em estado de espera.
//!    - Por que: Garante o uso eficiente do CPU, evitando que tarefas fiquem em "busy-wait" desnecessário.
//!
//! 3. `test_atomic_integrity`:
//!    - O que: Executar operações de add/sub atômicas em loop.
//!    - Por que: Assegura que o hardware está respeitando as garantias de atomicidade para flags e contadores de referência.
//!
//! 4. `test_reentrancy_deadlock`:
//!    - O que: Tentar adquirir o mesmo lock duas vezes na mesma thread e verificar se o sistema detecta ou lida com isso.
//!    - Por que: Previne travamentos fatais (deadlocks) causados por lógica de travamento circular.
