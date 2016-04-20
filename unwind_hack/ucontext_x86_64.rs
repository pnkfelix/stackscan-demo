// Taken from /usr/include/sys/ucontext.h
// (portion guarded by #ifdef __x86_64)

use libc::{uint16_t, uint32_t, uint64_t, c_ulong, c_ushort};
use super::sigset::__sigset_t;
use super::sigstack::stack_t;
const NGREG: usize = 23;

type gregset_t = [greg_t; NGREG];

#[repr(i64)] // #[repr(c_longlong)]
enum greg_t {
    R8 = 0, R9, R10, R11, R12, R13, R14, R15,
    RDI, RSI, RBP, RBX, RDX, RAX, RCX, RSP, RIP,
    EFL, CSGSFS, ERR, TRAPNO, OLDMASK, CR2,
}

#[repr(C)]
struct _libc_fpxreg {
    significand: [c_ushort; 4],
    exponent: c_ushort,
    padding: [c_ushort; 3],
}

#[repr(C)]
struct _libc_xmmreg {
    element: [uint32_t; 4]
}

#[repr(C)]
struct _libc_fpstate {
    cwd: uint16_t,
    swd: uint16_t,
    ftw: uint16_t,
    fop: uint16_t,
    rip: uint64_t,
    rdp: uint64_t,
    mxcsr: uint32_t,
    mxcr_mask: uint32_t,
    _st: [_libc_fpxreg; 8],
    _xmm: [_libc_xmmreg; 16],
    padding: [uint32_t; 24],
}

type fpregset_t = *const _libc_fpstate;

#[repr(C)]
struct mcontext_t {
    gregs: gregset_t,
    fpregs: fpregset_t,
    oldmask: c_ulong,
    cr2: c_ulong,
}

#[repr(C)]
pub struct ucontext_t {
    uc_flags: c_ulong,
    uc_link: *const ucontext_t,
    uc_stack: stack_t,
    uc_mcontext: mcontext_t,
    uc_sigmask: __sigset_t,
    __fpregs_mem: _libc_fpstate,
}
