//! # Syscall Entry (x86_64)
//!
//! Suporte à instrução `syscall` para chamadas de sistema.

use core::arch::asm;

const MSR_STAR: u32 = 0xC0000081;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_SFMASK: u32 = 0xC0000084;
const MSR_EFER: u32 = 0xC0000080;
const EFER_SCE: u64 = 1;

#[inline(always)]
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn rdmsr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

#[no_mangle]
pub extern "C" fn syscall_rust_entry(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    use crate::syscall::dispatch::table::{SYSCALL_TABLE, TABLE_SIZE};
    use crate::syscall::error::SysError;

    if num >= TABLE_SIZE {
        return SysError::InvalidSyscall.as_isize();
    }

    let args = crate::syscall::abi::SyscallArgs {
        num,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
    };

    match SYSCALL_TABLE[num] {
        Some(handler) => match handler(&args) {
            Ok(v) => v as isize,
            Err(e) => e.as_isize(),
        },
        None => SysError::NotImplemented.as_isize(),
    }
}

pub unsafe fn init() {
    crate::kdebug!("(Syscall) Configurando MSRs...");

    let efer = rdmsr(MSR_EFER);
    wrmsr(MSR_EFER, efer | EFER_SCE);

    // STAR: [63:48]=SYSRET base, [47:32]=SYSCALL base
    // SYSCALL: CS = base, SS = base+8
    // SYSRET:  SS = base+8, CS = base+16
    // GDT: 0x08=kernel_code, 0x10=kernel_data, 0x18=user_data, 0x20=user_code
    let star: u64 = (0x10u64 << 48) | (0x08u64 << 32);
    wrmsr(MSR_STAR, star);

    extern "C" {
        fn syscall_entry_asm();
    }
    wrmsr(MSR_LSTAR, syscall_entry_asm as u64);

    wrmsr(MSR_SFMASK, 0x200 | 0x400 | 0x100);

    crate::kinfo!("(Syscall) MSRs configurados");
}
