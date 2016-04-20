// Taken from /usr/include/sys/ucontext.h
// (non __x86_64 portion)

use libc::{c_ulong, c_ushort};

const NGREG: usize = 19;

type gregset_t = [greg_t; NGREG];

#[repr(c_int)]
enum greg_t {
    GS, FS, ES, DS, EDI, ESI, EBP, ESP, EBX, EDX, ECX, EAX,
    TRAPNO, ERR, EIP, CS, EFL, UESP, SS,
}

#[repr(C)]
struct _libc_fpreg {
    significand: [c_ushort; 4],
    exponent: c_ushort,
}

#[repr(C)]
struct _libc_fpstate {
    cw: c_ulong,
    sw: c_ulong,
    tag: c_ulong,
    ipoff: c_ulong,
    cssel: c_ulong,
    dataoff: c_ulong,
    _st: [_libc_fpreg; 8],
    status: c_ulong,
}

type fpregset_t = *mut _libc_fpstate;

struct mcontext_t {
    gregs: gregset_t,
    fpregs: fpregset_t,
    oldmask: c_ulong,
    cr2: c_ulong,
}

struct ucontext_t {
    uc_flags: c_ulong,
    uc_link: *mut ucontext_t,
    uc_stack: stack_t,
    uc_mcontext: mcontext_t,
    uc_sigmask: __sigset_t,
    __fpregs_mem: _libc_fpstate,
}
