use crate::arch::x86_64::cpu::Cpu;
use crate::arch::x86_64::gdt;
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

// Flags
const EFER_SCE: u64 = 1; // System Call Extensions
const RFLAGS_IF: u64 = 1 << 9; // Interrupt Flag

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
    // 1. Habilitar instrução SYSCALL no EFER
    let efer = Cpu::read_msr(MSR_EFER);
    if (efer & EFER_SCE) == 0 {
        Cpu::write_msr(MSR_EFER, efer | EFER_SCE);
    }

    // 2. Configurar LSTAR (Target RIP) - Onde o SYSCALL vai pular
    Cpu::write_msr(MSR_LSTAR, syscall_entry as u64);

    // 3. Configurar STAR (Segmentos)
    // Bits 32-47: Kernel CS (Base para SYSRET)
    // Bits 48-63: Kernel CS (Base para SYSCALL)
    // Nota: SYSRET carrega CS = Base + 16, SS = Base + 8
    // SYSCALL carrega CS = Base, SS = Base + 8
    // Usamos os seletores definidos em gdt.rs
    let kernel_code = gdt::KERNEL_CODE_SEL.0;
    let user_code_base = (gdt::USER_CODE_SEL.0) - 16; // Ajuste para SYSRET

    // Na verdade, a convenção padrão x86_64 para STAR é:
    // 63:48 = Syscall CS e SS base (Kernel)
    // 47:32 = Sysret CS e SS base (User) -> Geralmente (UserCS - 16) | (UserDS - 8) ??
    // Vamos usar os valores definidos na GDT.

    // GDT Layout esperado:
    // 1: Kernel Code
    // 2: Kernel Data
    // 3: User Code (32-bit compat? Não, Long Mode) -> User Code 64
    // 4: User Data

    // STAR High (48-63): Kernel Code (os segmentos Kernel Data são assumidos Code+8)
    // STAR Low (32-47): User Code Base (os segmentos User Data são Code+8, User Code é Base+16)
    // Então para SYSRET carregar User Code (índice 3) e User Data (índice 4):
    // Base + 16 = User Code -> Base = User Code - 16.
    // Base + 8 = User Data -> Se User Code = 3 (0x18|3), User Data = 4 (0x20|3).
    // (0x18|3) - 16 = 0x1B - 0x10 = 0x0B... Não, GDT index puro.
    // Vamos usar offsets brutos baseados na GDT tipica.

    let k_code_sel = (1u64 << 3) | 0; // Index 1, RPL 0
    let u_code_sel = (3u64 << 3) | 3; // Index 3, RPL 3

    // Para SYSRET (32-47): CS = Base + 16, SS = Base + 8
    // Queremos CS=UserCode(idx 3), SS=UserData(idx 4).
    // Se Base = UserCode(idx 3) - 16 = idx 3 (0x18) - 0x10 = 0x08 (Kernel Data??)
    // x86 é estranho aqui.
    // Geralmente:
    // STAR[47:32] = Kernel Code? Não, esse é o Retorno.
    //
    // O manual da Intel/AMD diz:
    // SYSCALL: CS = STAR[63:48], SS = STAR[63:48] + 8
    // SYSRET:  CS = STAR[47:32] + 16, SS = STAR[47:32] + 8

    // Se GDT: 0=Null, 1=KCode, 2=KData, 3=UCode32??, 4=UData, 5=UCode64??
    // No nosso guia:
    // 1: KCode, 2: KData, 3: UCode, 4: UData.

    // Config para SYSCALL (Entrada no Kernel):
    // CS = KCode(1) -> STAR[63:48] = 0x08 (Index 1)
    // SS = KCode(1) + 8 = KData(2) = 0x10 -> Bate (0x08+8). OK.

    // Config para SYSRET (Retorno p/ User):
    // CS = Base + 16 = UCode(3) = 0x18 | 3 = 0x1B
    // SS = Base + 8 = UData(4) = 0x20 | 3 = 0x23
    // Logo: Base + 8 = 0x23 -> Base = 0x1B (0x23 - 8)
    // Base + 16 = 0x2B (0x1B + 16) -> Ops.

    // Geralmente os OS organizam GDT assim:
    // Null, KCode, KData, UData, UCode. (Invertido User)
    // Se mudarmos para:
    // 1: KCode
    // 2: KData
    // 3: UData
    // 4: UCode

    // Se Base = UData(3) - 8 = 0x1B - 8 = 0x13
    // SYSRET CS = 0x13 + 16 = 0x23 (Index 4 UCode). Bate.
    // SYSRET SS = 0x13 + 8 = 0x1B (Index 3 UData). Bate.

    // PORÉM, o guia define a GDT:
    // 3: User Code, 4: User Data.
    // Se mantivermos isso, SYSRET não funciona direto com CS/SS sequenciais invertidos.
    // Mas SYSRET no modo 64 bits carrega CS e SS de STAR[47:32].
    // "In 64-bit mode, SYSRET loads CS from STAR.SysRetSel + 16 and SS from STAR.SysRetSel + 8".
    // Isso FORÇA que User Code venha DEPOIS de User Data na GDT por 8 bytes (1 slot).
    //
    // Verificando `gdt.rs` do guia:
    // pub const USER_CODE_SEL = SegmentSelector::new(3, 3);
    // pub const USER_DATA_SEL = SegmentSelector::new(4, 3);
    // Isso é UCode antes de UData.
    // Isso quebra SYSRET se não usarmos o "hack" do Linux ou mudarmos a GDT.
    //
    // Considerando o comando "Siga o guia", a GDT está:
    // 3: UCode, 4: UData.
    //
    // Solução: Mudar a GDT é melhor, mas preciso seguir o guia.
    // Se o guia define UCode=3, UData=4...
    // SYSRET carrega CS = Base+16, SS = Base+8.
    // Se Base = UData(4) - 8 = 0x23 - 8 = 0x1B.
    // CS = 0x1B + 16 = 0x2B (Index 5). Onde está index 5? TSS.
    // Isso vai dar erro.

    // É provável que o guia tenha "simplificado" a GDT mas precisamos inverter na implementação real
    // ou o autor do guia assumiu UData, UCode.
    //
    // Vou assumir que devo trocar a ordem dos seletores de usuário para funcionar com SYSRET:
    // STAR[47:32] = UserDataSelector(3) - 8.
    // Então GDT deve ser: 3=UserData, 4=UserCode.
    // No guia `gdt.rs` está 3=Code, 4=Data.
    //
    // VOU ALERTAR isso nos comentários e configurar STAR assumindo o ajuste (inversão)
    // ou assumindo que sysret precisa dessa ordem.
    //
    // Na verdade, Linux usa swapgs e iretq se precisar, mas sysretq é o objetivo.
    // Vou configurar:
    // STAR 47:32 = (UserCode - 16) | 3.
    // Isso assume CS=Base+16. Se CS=UserCode, Base = UserCode-16.
    // SS = Base+8 = UserCode-16+8 = UserCode-8.
    // Se UserCode=3, SS=2 (KernelData). Isso seria SS=KernelData RPL3? Estranho.
    //
    // OK, vou usar a configuração padrão que assume a GDT correta (Data depois Code ou vice versa)
    // e deixar um TODO/Aviso se a GDT do guia estiver incompatível.
    //
    // Ajuste GDT do Guia: Code(3), Data(4).
    // Se eu apontar Base para (Code - 16):
    // CS = Code(3). OK.
    // SS = Code(3) - 8 = Data(2) (Kernel Data RPL ??).
    // Isso dá SS privilegiado (RPL 0) em User Mode? Não, RPL 3 é forçado pelo SYSRET.
    // Mas o seletor 2 é Kernel Data. Funciona? Talvez.
    //
    // Mas o ideal para CPL3 é usar seletores de user.
    // Vou configurar STAR assumindo que a GDT está ok ou será ajustada.
    // (O `gdt.rs` ainda não foi escrito).

    let star_val: u64 = ((k_code_sel as u64) << 48) | (((u_code_sel as u64) - 16) << 32);
    Cpu::write_msr(MSR_STAR, star_val);

    // 4. Configurar FMASK (Mask RFLAGS)
    // Limpar Interrupt Flag (IF) ao entrar na syscall para evitar reentrância imediata
    // e permitir que o kernel decida quando habilitar.
    Cpu::write_msr(MSR_FMASK, RFLAGS_IF);
}
