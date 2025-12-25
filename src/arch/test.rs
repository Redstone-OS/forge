//! Testes da Camada de Abstração de Hardware (Arch)
//!
//! # Por que testar?
//! A camada arch é a base de tudo. Se a GDT estiver errada, o kernel falha ao trocar de Anel (Privilégio).
//! Se a IDT falhar, qualquer interrupção de hardware ou exceção (como Page Fault) causará um Triple Fault.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_gdt_integrity`:
//!    - O que: Verificar se os seletores de segmento (Kernel Code/Data, User Code/Data) estão nos offsets corretos.
//!    - Por que: Garante que a segmentação x86_64 está configurada conforme o padrão do Redstone OS.
//!
//! 2. `test_idt_handlers`:
//!    - O que: Disparar uma interrupção de software (int 3) e verificar se o handler de breakpoint é chamado.
//!    - Por que: Valida que a IDT está carregada e que o kernel consegue desviar o fluxo para os handlers de exceção.
//!
//! 3. `test_tss_switching`:
//!    - O que: Verificar se a TSS (Task State Segment) contém o ponteiro para a Pilha de Privilégio (RSP0).
//!    - Por que: Fundamental para que o hardware saiba para onde trocar a stack quando ocorre uma interrupção em Ring 3.
//!
//! 4. `test_msr_consistency`:
//!    - O que: Ler os registradores MSR (especificamente STAR, LSTAR e SFMASK).
//!    - Por que: Garante que o mecanismo de `syscall/sysret` está configurado para permitir a transição rápida entre User e Kernel.
