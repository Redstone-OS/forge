//! # Syscall Dispatcher
//!
//! O ponto central onde o kernel atende pedidos do userspace.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Dispatching:** Roteia o número da syscall (`RAX`) para a função rust correspondente.
//! - **Argument decoding:** Extrai argumentos dos registradores (`RDI`, `RSI`, `RDX`...).
//! - **Safety Boundary:** É a **primeira linha de defesa** do kernel. Tudo que vem daqui é não-confiável.
//!
//! ## 🏗️ Arquitetura: Synchronous Handler
//! - A função `syscall_dispatcher` é chamada pelo trampoline assembly (`syscall_entry`).
//! - Ela roda no contexto da thread atual, mas em Ring 0 (Kernel Mode).
//! - O retorno é escrito em `context.rax`, que o assembly restaurará antes de `sysret`.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Register Mapping:** Mapeamento claro dos registradores SysV ABI para variáveis locais.
//! - **Centralized Handling:** Um único `match` facilita instrumentação e debugging de todas as chamadas.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Unsafe Pointer Dereference:** O código acessa `ptr` (ponteiro de usuário) DIRETAMENTE!
//!   - *Vulnerabilidade:* Se o usuário passar um endereço de kernel (ex: `0xFFFF...`), o kernel vai ler/escrever sua própria memória, permitindo **Privilege Escalation** ou **Crash**.
//!   - *Correção:* É OBRIGATÓRIO usar funções como `copy_from_user` que verificam limites (`ptr < USER_MAX_ADDR`).
//! - **Blocking I/O:** `SYS_WRITE` em console serial bloqueia a CPU. Se o buffer serial encher, o sistema trava.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Security)** Implementar **User Access Validation**.
//!   - *Meta:* Criar `UserPtr<T>` que encapsula verificação de range (0..UserMax).
//! - [ ] **TODO: (Feature)** Expandir Tabela de Syscalls.
//!   - *Necessário:* `mmap`, `spawn`, `ipc_send`, `ipc_recv`.
//! - [ ] **TODO: (Debug)** Adicionar **Strace** (Syscall Tracing).
//!   - *Meta:* Logar cada entrada/saída de syscall se uma flag de debug estiver ativa na Task.
//!
//! --------------------------------------------------------------------------------
//!
//! Handler de Syscalls chamado pelo Assembly.
//!
//! # Arguments
//! * `context`: Ponteiro para o stack frame com todos registradores salvos.
//!
//!
use crate::arch::x86_64::idt::ContextFrame;
use core::ffi::c_void;

// Syscall Numbers
const SYS_WRITE: u64 = 1;
const SYS_YIELD: u64 = 158; // Exemplo

#[no_mangle]
pub extern "C" fn syscall_dispatcher(context: &mut ContextFrame) {
    let syscall_num = context.rax;
    let arg1 = context.rdi;
    let arg2 = context.rsi;
    let arg3 = context.rdx;

    match syscall_num {
        SYS_WRITE => {
            let fd = arg1;
            let ptr = arg2 as *const u8;
            let len = arg3 as usize;

            crate::ktrace!("(Sys) sys_write: fd=", fd);
            crate::klog!(" ptr=", ptr as u64, " len=", len as u64);
            crate::knl!();

            if fd == 1 {
                // STDOUT
                if ptr.is_null() {
                    crate::kwarn!("(Sys) sys_write: Ponteiro nulo recebido");
                    context.rax = -1i64 as u64;
                    return;
                }

                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

                // Tenta converter para UTF-8 string
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::klog!(s);
                } else {
                    for &b in slice {
                        crate::drivers::serial::emit(b);
                    }
                }

                context.rax = len as u64; // Retorna bytes escritos
            } else {
                crate::kdebug!("(Sys) sys_write: FD não suportado FD=", fd);
                context.rax = -1i64 as u64; // EBADF
            }
        }
        SYS_YIELD => {
            crate::ktrace!("(Sys) sys_yield: Cedendo tempo de CPU voluntariamente");
            crate::sched::scheduler::yield_now();
            context.rax = 0;
        }
        _ => {
            crate::kwarn!("(Sys) Chamada inesperada: syscall num=", syscall_num);
            context.rax = -1i64 as u64; // ENOSYS
        }
    }
}
