#![allow(bad_asm_style)]
use crate::arch::x86_64::cpu::Cpu;
/// Arquivo: x86_64/syscall.rs
///
/// Propósito: Configuração e tratamento de chamadas de sistema (System Calls).
///
/// Detalhes de Implementação:
/// - Inicializa MSRs (STAR, LSTAR, FMASK) para instruções SYSCALL/SYSRET.
/// - Define a estrutura `TrapFrame` que espelha o estado salvo pelo assembly (`syscall.s`).
/// - Implementa o `syscall_dispatcher` que roteia chamadas baseado em RAX.
///
/// MSRs Importantes:
/// - EFER (0xC0000080): Bit 0 (SCE) habilita SYSCALL.
/// - STAR (0xC0000081): Seletores de segmento para User/Kernel.
/// - LSTAR (0xC0000082): Endereço de destino (RIP) do SYSCALL.
/// - FMASK (0xC0000084): Máscara de RFLAGS (limpa Interrupt Flag).
use core::arch::global_asm;

// Incluir o trampolim assembly
global_asm!(include_str!("syscall.s"));

// Constantes MSR
const MSR_EFER: u32 = 0xC0000080;
const MSR_STAR: u32 = 0xC0000081;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_FMASK: u32 = 0xC0000084;
const MSR_KERNEL_GS_BASE: u32 = 0xC0000102;
const MSR_GS_BASE: u32 = 0xC0000101;

// Flags
const EFER_SCE: u64 = 1; // System Call Extensions
const RFLAGS_IF: u64 = 1 << 9; // Interrupt Flag

/// Estrutura per-CPU para syscall.
/// syscall.s acessa via gs:[0] (user_rsp) e gs:[8] (kernel_rsp).
#[repr(C)]
pub struct SyscallStack {
    /// RSP do usuário salvo pelo handler (gs:[0])
    pub user_rsp: u64,
    /// Kernel stack pointer para usar durante syscall (gs:[8])
    pub kernel_rsp: u64,
}

/// Stack de syscall para BSP (CPU 0)
/// TODO: Tornar per-CPU com array para SMP
static mut SYSCALL_STACK: SyscallStack = SyscallStack {
    user_rsp: 0,
    kernel_rsp: 0,
};

/// Estrutura que representa o estado salvo dos registradores na stack.
/// Deve corresponder EXATAMENTE à ordem de push em `syscall.s`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    // Registradores salvos manualmente (Callee-saved + Args)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    // Frame de Hardware / Simulado
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

extern "C" {
    /// Símbolo definido em `syscall.s`
    fn syscall_entry();
}

/// Inicializa o mecanismo de System Calls (instruções SYSCALL/SYSRET).
///
/// # Safety
///
/// Esta função deve ser chamada apenas uma vez durante a inicialização do BSP.
/// Escreve em MSRs específicos da CPU.
pub unsafe fn init() {
    // 1. Habilitar instrução SYSCALL e bit NXE no EFER
    const EFER_NXE: u64 = 1 << 11;
    let efer = Cpu::read_msr(MSR_EFER);
    let mut new_efer = efer | EFER_SCE;

    // SEMPRE habilitar NXE se disponível (x86_64 requer suporte mas precisa habilitar)
    new_efer |= EFER_NXE;

    if new_efer != efer {
        Cpu::write_msr(MSR_EFER, new_efer);
    }

    // 2. Configurar LSTAR (Target RIP) - Onde o SYSCALL vai pular
    Cpu::write_msr(MSR_LSTAR, syscall_entry as u64);

    // 3. Configurar STAR (Segmentos)
    //
    // STAR MSR Layout:
    // - Bits 63:48 = SYSCALL CS base (Kernel) → CS = STAR[63:48], SS = STAR[63:48] + 8
    // - Bits 47:32 = SYSRET CS/SS base (User) → CS = STAR[47:32] + 16, SS = STAR[47:32] + 8
    //
    // Nossa GDT (após inversão para SYSRET):
    // - Index 1: Kernel Code (0x08)
    // - Index 2: Kernel Data (0x10)
    // - Index 3: User Data  (0x1B com RPL 3)
    // - Index 4: User Code  (0x23 com RPL 3)
    //
    // Para SYSCALL (entrada no kernel):
    // - CS = STAR[63:48] = 0x08 (Kernel Code)
    // - SS = 0x08 + 8 = 0x10 (Kernel Data) ✓
    //
    // Para SYSRET (retorno ao user):
    // - Queremos CS = 0x23 (User Code, idx 4) e SS = 0x1B (User Data, idx 3)
    // - CS = Base + 16 = 0x23 → Base = 0x13
    // - SS = Base + 8  = 0x1B → Base = 0x13 ✓
    //
    // Portanto: STAR[47:32] = 0x13 (que é User Data sem RPL menos 8)

    let syscall_kcode_base: u64 = 0x08; // Kernel Code (Index 1, RPL 0)
    let sysret_base: u64 = 0x13; // User Data (0x1B) - 8 = 0x13

    let star_val: u64 = (sysret_base << 48) | (syscall_kcode_base << 32);
    Cpu::write_msr(MSR_STAR, star_val);

    // 4. Configurar FMASK (Mask RFLAGS)
    // Limpar Interrupt Flag (IF) ao entrar na syscall para evitar reentrância imediata
    // e permitir que o kernel decida quando habilitar.
    Cpu::write_msr(MSR_FMASK, RFLAGS_IF);

    // 5. Configurar GS Base para syscalls
    // INICIALIZAÇÃO CRÍTICA:
    // Estamos em Kernel Mode. O 'Active' GS Base deve apontar para as estruturas do Kernel (SYSCALL_STACK).
    // O 'Shadow' (KERNEL_GS_BASE MSR) deve guardar o valor do GS do usuário (ou 0 inicialmente).
    // Quando 'iretq_restore' executar 'swapgs', ele trocará Active(Kernel) <-> Shadow(User).
    // Assim, o User Mode rodará com GS=User e MSR=Kernel.
    // Quando 'syscall' ocorrer, 'swapgs' trocará Active(User) <-> Shadow(Kernel).
    let syscall_stack_addr = core::ptr::addr_of!(SYSCALL_STACK) as u64;

    // Configurar Active GS Base (para uso imediato no Kernel)
    Cpu::write_msr(MSR_GS_BASE, syscall_stack_addr);

    // Configurar Shadow GS Base (para ser carregado via swapgs ao ir para user)
    Cpu::write_msr(MSR_KERNEL_GS_BASE, 0);

    crate::kinfo!("(Syscall) GS_BASE inicializado:", syscall_stack_addr);
    crate::kinfo!("(Syscall) KERNEL_GS_BASE (Shadow) inicializado com 0");
}

/// Configura o kernel RSP para a task atual.
/// Deve ser chamado durante context switch para atualizar o RSP usado em syscalls.
///
/// # Safety
///
/// O kernel_stack deve ser um endereço válido e mapeado.
pub unsafe fn set_kernel_rsp(kernel_stack: u64) {
    SYSCALL_STACK.kernel_rsp = kernel_stack;
}
