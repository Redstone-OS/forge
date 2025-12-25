//! Testes do Escalonador (Scheduler)
//!
//! # Por que testar?
//! O Scheduler é o "coração" do sistema multitarefa. Erros aqui causam congelamentos, lentidão extrema
//! ou o temido "Kernel Panic" por corrupção de stack durante a troca de contexto.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_context_switch_preemption`:
//!    - O que: Criar duas tarefas e garantir que o Timer (PIT) as alterna periodicamente.
//!    - Por que: Se o Preemptive Scheduling falhar, uma tarefa infinita travará o computador inteiro.
//!
//! 2. `test_task_state_transitions`:
//!    - O que: Verificar se uma tarefa passa corretamente de Running -> Blocked -> Ready.
//!    - Por que: Garante que tarefas esperando por I/O não consumam CPU e voltem a rodar assim que o dado estiver pronto.
//!
//! 3. `test_priority_queue`:
//!    - O que: Criar tarefas com prioridades diferentes e validar se as de maior prioridade rodam com mais frequência.
//!    - Por que: Fundamental para garantir a responsividade de drivers críticos frente a processos de usuário pesados.
//!
//! 4. `test_stack_overflow_protection`:
//!    - O que: Criar uma tarefa com recursão infinita e verificar se o kernel detecta o estouro da stack (via Guard Pages).
//!    - Por que: Previne que um bug numa tarefa corrompa dados de outras tarefas ou do próprio kernel.
