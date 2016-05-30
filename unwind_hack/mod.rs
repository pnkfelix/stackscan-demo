//! This file is a port of the libunwind.h header declarations of the
//! LLVM fork of libunwind.

use libc::{self, c_char, c_double, c_int, c_void, size_t};
use std::mem;
use std::ffi::CStr;

#[repr(i32)]
#[derive(Debug)]
pub enum UnwindError {
    /// unspecified (general) error
    Unspec       = -6540,
    /// out of memory
    NoMem        = -6541,
    /// bad register number
    BadReg       = -6542,
    /// attempt to write read-only register
    ReadOnlyReg  = -6543,
    /// stop unwinding
    StopUnwind   = -6544,
    /// invalid IP
    InvalidIP    = -6545,
    /// bad frame
    BadFrame     = -6546,
    /// unsupported operation or bad value
    Invalid      = -6547,
    /// unwind info has unsupported version
    BadVersion   = -6548,
    /// no unwind info found
    NoInfo       = -6549,
}
#[repr(i32)]
#[derive(Debug)]
pub enum ErrorCode {
    /// no error
    SUCCESS      = 0,
    /// unspecified (general) error
    UNSPEC       = UnwindError::Unspec as i32,
    /// out of memory
    NOMEM        = UnwindError::NoMem as i32,
    /// bad register number
    BADREG       = UnwindError::BadReg as i32,
    /// attempt to write read-only register
    READONLYREG  = UnwindError::ReadOnlyReg as i32,
    /// stop unwinding
    STOPUNWIND   = UnwindError::StopUnwind as i32,
    /// invalid IP
    INVALIDIP    = UnwindError::InvalidIP as i32,
    /// bad frame
    BADFRAME     = UnwindError::BadFrame as i32,
    /// unsupported operation or bad value
    INVAL        = UnwindError::Invalid as i32,
    /// unwind info has unsupported version
    BADVERSION   = UnwindError::BadVersion as i32,
    /// no unwind info found
    NOINFO       = UnwindError::NoInfo as i32,
}
pub const UNW_ESUCCESS: ErrorCode = ErrorCode::SUCCESS;
pub const UNW_EUNSPEC: ErrorCode = ErrorCode::UNSPEC;
pub const UNW_ENOMEM: ErrorCode = ErrorCode::NOMEM;
pub const UNW_EBADREG: ErrorCode = ErrorCode::BADREG;
pub const UNW_EREADONLYREG: ErrorCode = ErrorCode::READONLYREG;
pub const UNW_ESTOPUNWIND: ErrorCode = ErrorCode::STOPUNWIND;
pub const UNW_EINVALIDIP: ErrorCode = ErrorCode::INVALIDIP;
pub const UNW_EBADFRAME: ErrorCode = ErrorCode::BADFRAME;
pub const UNW_EINVAL: ErrorCode = ErrorCode::INVAL;
pub const UNW_EBADVERSION: ErrorCode = ErrorCode::BADVERSION;
pub const UNW_ENOINFO: ErrorCode = ErrorCode::NOINFO;

const UNW_CONTEXT_LEN: usize = 128;

#[repr(C)]
pub struct unw_context_t { data: [u64; UNW_CONTEXT_LEN] }

const UNW_CURSOR_LEN: usize = 140;

#[repr(C)]
pub struct unw_cursor_t { data: [u64; UNW_CURSOR_LEN] }

#[repr(C)]
pub struct unw_addr_space { hidden: usize }
pub type unw_addr_space_t = *const unw_addr_space;

pub type unw_regnum_t = c_int;

#[cfg(target_arch="arm")]
pub type unw_word_t = u32;
#[cfg(target_arch="arm")]
pub type unw_fpreg_t = u64;

#[cfg(not(target_arch="arm"))]
pub type unw_word_t = u64;
#[cfg(not(target_arch="arm"))]
pub type unw_fpreg_t = f64;

pub struct unw_proc_info_t {
    /// start address of function
    start_ip: unw_word_t,
    /// address after end of function
    end_ip: unw_word_t,
    /// address of language specific data area or zero if not used
    lsda: unw_word_t,
    /// personality routine, or zero if not used
    handler: unw_word_t,
    /// not used
    gp: unw_word_t,
    /// not used
    flags: unw_word_t,
    /// compact unwind encoding, or zero if none
    format: u32,
    /// size of dwarf unwind info, or zero if none
    unwind_info_size: u32,
    /// address of dwarf unwind info, or zero
    unwind_info: unw_word_t,
    /// mach_header of mach-o image containing func
    extra: unw_word_t,
}

#[link(name="unwind")]
extern {
    fn unw_getcontext(ucp: *mut unw_context_t) -> c_int;
    fn unw_init_local(c: *mut unw_cursor_t, ctxt: *const unw_context_t) -> c_int;
    fn unw_step(cp: *mut unw_cursor_t) -> c_int;
    fn unw_get_reg(cp: *const unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t) -> c_int;
    fn unw_get_fpreg(cp: *const unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_fpreg_t) -> c_int;
    fn unw_resume(cp: *const unw_cursor_t) -> c_int;
    fn unw_regname(regnum: unw_regnum_t) -> *const c_char;
    fn unw_get_proc_info(cp: *const unw_cursor_t, pip: *mut unw_proc_info_t) -> c_int;
    fn unw_is_fpreg(r: unw_regnum_t) -> c_int;
    fn unw_is_signal_frame(cp: *const unw_cursor_t) -> c_int;
    fn unw_get_proc_name(cp: *const unw_cursor_t, bufp: *mut c_char, len: size_t, offp: *mut unw_word_t) -> c_int;

    static unw_local_addr_space: unw_addr_space_t;
}

macro_rules! enum_as_consts {
    ($Enum:ident, $($UNW_NAME:ident = $Variant:ident),*) => {
        $( pub const $UNW_NAME: unw_regnum_t = $Enum::$Variant as unw_regnum_t; )*
    }
}

enum ArchIndepReg { IP = -1, SP = -2, }
enum_as_consts!(ArchIndepReg, UNW_REG_IP = IP, UNW_REG_SP = SP);

enum X86Reg { EAX = 0, ECX = 1, EDX = 2, EBX = 3, EBP = 4, ESP = 5, ESI = 6, EDI = 7, }
enum_as_consts!(X86Reg,
                UNW_X86_EAX = EAX,
                UNW_X86_ECX = ECX,
                UNW_X86_EDX = EDX,
                UNW_X86_EBX = EBX,
                UNW_X86_ESP = ESP,
                UNW_X86_ESI = ESI,
                UNW_X86_EDI = EDI);

enum X86_64Reg { RAX = 0, RDX = 1, RCX = 2, RBX = 3, RSI = 4, RDI = 5, RBP = 6, RSP = 7,
                 R8  = 8, R9  = 9, R10 = 10, R11 = 11, R12 = 12, R13 = 13, R14 = 14, R15 = 15 }
enum_as_consts!(X86_64Reg,
                UNW_X86_64_RAX = RAX,
                UNW_X86_64_RDX = RDX,
                UNW_X86_64_RCX = RCX,
                UNW_X86_64_RBX = RBX,
                UNW_X86_64_RSI = RSI,
                UNW_X86_64_RDI = RDI,
                UNW_X86_64_RBP = RBP,
                UNW_X86_64_RSP = RSP,
                UNW_X86_64_R8  = R8 ,
                UNW_X86_64_R9  = R9 ,
                UNW_X86_64_R10 = R10,
                UNW_X86_64_R11 = R11,
                UNW_X86_64_R12 = R12,
                UNW_X86_64_R13 = R13,
                UNW_X86_64_R14 = R14,
                UNW_X86_64_R15 = R15);

// 32-bit ppc register numbers
enum PPCReg { R0  = 0,  R1  = 1,  R2  = 2,  R3  = 3,  R4  = 4,  R5  = 5,  R6  = 6,  R7  = 7,
              R8  = 8,  R9  = 9,  R10 = 10, R11 = 11, R12 = 12, R13 = 13, R14 = 14, R15 = 15,
              R16 = 16, R17 = 17, R18 = 18, R19 = 19, R20 = 20, R21 = 21, R22 = 22, R23 = 23,
              R24 = 24, R25 = 25, R26 = 26, R27 = 27, R28 = 28, R29 = 29, R30 = 30, R31 = 31,
              F0  = 32, F1  = 33, F2  = 34, F3  = 35, F4  = 36, F5  = 37, F6  = 38, F7  = 39,
              F8  = 40, F9  = 41, F10 = 42, F11 = 43, F12 = 44, F13 = 45, F14 = 46, F15 = 47,
              F16 = 48, F17 = 49, F18 = 50, F19 = 51, F20 = 52, F21 = 53, F22 = 54, F23 = 55,
              F24 = 56, F25 = 57, F26 = 58, F27 = 59, F28 = 60, F29 = 61, F30 = 62, F31 = 63,
              MQ  = 64, LR  = 65, CTR = 66, AP  = 67, CR0 = 68, CR1 = 69, CR2 = 70, CR3 = 71,
              CR4 = 72, CR5 = 73, CR6 = 74, CR7 = 75, XER = 76,
              V0  = 77, V1  = 78, V2  = 79, V3  = 80, V4  = 81, V5  = 82, V6  = 83, V7  = 84,
              V8  = 85, V9  = 86, V10 = 87, V11 = 88, V12 = 89, V13 = 90, V14 = 91, V15 = 92,
              V16 = 93, V17 = 94, V18 = 95, V19 = 96, V20 = 97, V21 = 98, V22 = 99, V23 = 100,
              V24 = 101,V25 = 102,V26 = 103,V27 = 104,V28 = 105,V29 = 106,V30 = 107,V31 = 108,
              VRSAVE  = 109, VSCR    = 110, SPE_ACC = 111, SPEFSCR = 112
}
enum_as_consts!(PPCReg,
                UNW_PPC_R0  = R0,
                UNW_PPC_R1  = R1,
                UNW_PPC_R2  = R2,
                UNW_PPC_R3  = R3,
                UNW_PPC_R4  = R4,
                UNW_PPC_R5  = R5,
                UNW_PPC_R6  = R6,
                UNW_PPC_R7  = R7,
                UNW_PPC_R8  = R8,
                UNW_PPC_R9  = R9,
                UNW_PPC_R10 = R10,
                UNW_PPC_R11 = R11,
                UNW_PPC_R12 = R12,
                UNW_PPC_R13 = R13,
                UNW_PPC_R14 = R14,
                UNW_PPC_R15 = R15,
                UNW_PPC_R16 = R16,
                UNW_PPC_R17 = R17,
                UNW_PPC_R18 = R18,
                UNW_PPC_R19 = R19,
                UNW_PPC_R20 = R20,
                UNW_PPC_R21 = R21,
                UNW_PPC_R22 = R22,
                UNW_PPC_R23 = R23,
                UNW_PPC_R24 = R24,
                UNW_PPC_R25 = R25,
                UNW_PPC_R26 = R26,
                UNW_PPC_R27 = R27,
                UNW_PPC_R28 = R28,
                UNW_PPC_R29 = R29,
                UNW_PPC_R30 = R30,
                UNW_PPC_R31 = R31,
                UNW_PPC_F0  = F0,
                UNW_PPC_F1  = F1,
                UNW_PPC_F2  = F2,
                UNW_PPC_F3  = F3,
                UNW_PPC_F4  = F4,
                UNW_PPC_F5  = F5,
                UNW_PPC_F6  = F6,
                UNW_PPC_F7  = F7,
                UNW_PPC_F8  = F8,
                UNW_PPC_F9  = F9,
                UNW_PPC_F10 = F10,
                UNW_PPC_F11 = F11,
                UNW_PPC_F12 = F12,
                UNW_PPC_F13 = F13,
                UNW_PPC_F14 = F14,
                UNW_PPC_F15 = F15,
                UNW_PPC_F16 = F16,
                UNW_PPC_F17 = F17,
                UNW_PPC_F18 = F18,
                UNW_PPC_F19 = F19,
                UNW_PPC_F20 = F20,
                UNW_PPC_F21 = F21,
                UNW_PPC_F22 = F22,
                UNW_PPC_F23 = F23,
                UNW_PPC_F24 = F24,
                UNW_PPC_F25 = F25,
                UNW_PPC_F26 = F26,
                UNW_PPC_F27 = F27,
                UNW_PPC_F28 = F28,
                UNW_PPC_F29 = F29,
                UNW_PPC_F30 = F30,
                UNW_PPC_F31 = F31,
                UNW_PPC_MQ  = MQ ,
                UNW_PPC_LR  = LR ,
                UNW_PPC_CTR = CTR,
                UNW_PPC_AP  = AP ,
                UNW_PPC_CR0 = CR0,
                UNW_PPC_CR1 = CR1,
                UNW_PPC_CR2 = CR2,
                UNW_PPC_CR3 = CR3,
                UNW_PPC_CR4 = CR4,
                UNW_PPC_CR5 = CR5,
                UNW_PPC_CR6 = CR6,
                UNW_PPC_CR7 = CR7,
                UNW_PPC_XER = XER,
                UNW_PPC_V0  = V0 ,
                UNW_PPC_V1  = V1 ,
                UNW_PPC_V2  = V2 ,
                UNW_PPC_V3  = V3 ,
                UNW_PPC_V4  = V4 ,
                UNW_PPC_V5  = V5 ,
                UNW_PPC_V6  = V6 ,
                UNW_PPC_V7  = V7 ,
                UNW_PPC_V8  = V8 ,
                UNW_PPC_V9  = V9 ,
                UNW_PPC_V10 = V10,
                UNW_PPC_V11 = V11,
                UNW_PPC_V12 = V12,
                UNW_PPC_V13 = V13,
                UNW_PPC_V14 = V14,
                UNW_PPC_V15 = V15,
                UNW_PPC_V16 = V16,
                UNW_PPC_V17 = V17,
                UNW_PPC_V18 = V18,
                UNW_PPC_V19 = V19,
                UNW_PPC_V20 = V20,
                UNW_PPC_V21 = V21,
                UNW_PPC_V22 = V22,
                UNW_PPC_V23 = V23,
                UNW_PPC_V24 = V24,
                UNW_PPC_V25 = V25,
                UNW_PPC_V26 = V26,
                UNW_PPC_V27 = V27,
                UNW_PPC_V28 = V28,
                UNW_PPC_V29 = V29,
                UNW_PPC_V30 = V30,
                UNW_PPC_V31 = V31,
                UNW_PPC_VRSAVE  = VRSAVE,
                UNW_PPC_VSCR    = VSCR,
                UNW_PPC_SPE_ACC = SPE_ACC,
                UNW_PPC_SPEFSCR = SPEFSCR);

// 64-bit ARM64 registers
enum ARM64Reg { X0  = 0, X1  = 1, X2  = 2, X3  = 3, X4  = 4, X5  = 5, X6  = 6, X7  = 7, 
                X8  = 8, X9  = 9, X10 = 10,X11 = 11,X12 = 12,X13 = 13,X14 = 14,X15 = 15,
                X16 = 16,X17 = 17,X18 = 18,X19 = 19,X20 = 20,X21 = 21,X22 = 22,X23 = 23,
                X24 = 24,X25 = 25,X26 = 26,X27 = 27,X28 = 28,FP  = 29,LR  = 30,SP  = 31,
                // reserved block
                D0  = 64,D1  = 65,D2  = 66,D3  = 67,D4  = 68,D5  = 69,D6  = 70,D7  = 71,
                D8  = 72,D9  = 73,D10 = 74,D11 = 75,D12 = 76,D13 = 77,D14 = 78,D15 = 79,
                D16 = 80,D17 = 81,D18 = 82,D19 = 83,D20 = 84,D21 = 85,D22 = 86,D23 = 87,
                D24 = 88,D25 = 89,D26 = 90,D27 = 91,D28 = 92,D29 = 93,D30 = 94,D31 = 95,
}
enum_as_consts!(ARM64Reg,
                UNW_ARM64_X0  = X0,
                UNW_ARM64_X1  = X1,
                UNW_ARM64_X2  = X2,
                UNW_ARM64_X3  = X3,
                UNW_ARM64_X4  = X4,
                UNW_ARM64_X5  = X5,
                UNW_ARM64_X6  = X6,
                UNW_ARM64_X7  = X7,
                UNW_ARM64_X8  = X8,
                UNW_ARM64_X9  = X9,
                UNW_ARM64_X10 = X10,
                UNW_ARM64_X11 = X11,
                UNW_ARM64_X12 = X12,
                UNW_ARM64_X13 = X13,
                UNW_ARM64_X14 = X14,
                UNW_ARM64_X15 = X15,
                UNW_ARM64_X16 = X16,
                UNW_ARM64_X17 = X17,
                UNW_ARM64_X18 = X18,
                UNW_ARM64_X19 = X19,
                UNW_ARM64_X20 = X20,
                UNW_ARM64_X21 = X21,
                UNW_ARM64_X22 = X22,
                UNW_ARM64_X23 = X23,
                UNW_ARM64_X24 = X24,
                UNW_ARM64_X25 = X25,
                UNW_ARM64_X26 = X26,
                UNW_ARM64_X27 = X27,
                UNW_ARM64_X28 = X28,
                UNW_ARM64_X29 = FP,
                UNW_ARM64_FP  = FP,
                UNW_ARM64_X30 = LR,
                UNW_ARM64_LR  = LR,
                UNW_ARM64_X31 = SP,
                UNW_ARM64_SP  = SP,
                // reserved block
                UNW_ARM64_D0  = D0,
                UNW_ARM64_D1  = D1,
                UNW_ARM64_D2  = D2,
                UNW_ARM64_D3  = D3,
                UNW_ARM64_D4  = D4,
                UNW_ARM64_D5  = D5,
                UNW_ARM64_D6  = D6,
                UNW_ARM64_D7  = D7,
                UNW_ARM64_D8  = D8,
                UNW_ARM64_D9  = D9,
                UNW_ARM64_D10 = D10,
                UNW_ARM64_D11 = D11,
                UNW_ARM64_D12 = D12,
                UNW_ARM64_D13 = D13,
                UNW_ARM64_D14 = D14,
                UNW_ARM64_D15 = D15,
                UNW_ARM64_D16 = D16,
                UNW_ARM64_D17 = D17,
                UNW_ARM64_D18 = D18,
                UNW_ARM64_D19 = D19,
                UNW_ARM64_D20 = D20,
                UNW_ARM64_D21 = D21,
                UNW_ARM64_D22 = D22,
                UNW_ARM64_D23 = D23,
                UNW_ARM64_D24 = D24,
                UNW_ARM64_D25 = D25,
                UNW_ARM64_D26 = D26,
                UNW_ARM64_D27 = D27,
                UNW_ARM64_D28 = D28,
                UNW_ARM64_D29 = D29,
                UNW_ARM64_D30 = D30,
                UNW_ARM64_D31 = D31);

// 32-bit ARM registers. Numbers match DWARF for ARM spec #3.1 Table 1.
// Naming scheme uses recommendations given in Note 4 for VFP-v2 and VFP-v3.
// In this scheme, even though the 64-bit floating point registers D0-D31
// overlap physically with the 32-bit floating pointer registers S0-S31,
// they are given a non-overlapping range of register numbers.
//
// Commented out ranges are not preserved during unwinding.
enum ARM32Reg { R0  = 0,  R1  = 1,  R2  = 2,   R3  = 3,  R4  = 4,  R5  = 5, R6  = 6,  R7  = 7,
                R8  = 8,  R9  = 9,  R10 = 10, R11 = 11, R12 = 12, SP  = 13, LR  = 14, IP  = 15,
                // 16-63 -- OBSOLETE. Used in VFP1 to represent both S0-S31 and D0-D31.
                S0  = 64, S1  = 65, S2  = 66, S3  = 67, S4  = 68, S5  = 69, S6  = 70, S7  = 71,
                S8  = 72, S9  = 73, S10 = 74, S11 = 75, S12 = 76, S13 = 77, S14 = 78, S15 = 79, 
                S16 = 80, S17 = 81, S18 = 82, S19 = 83, S20 = 84, S21 = 85, S22 = 86, S23 = 87, 
                S24 = 88, S25 = 89, S26 = 90, S27 = 91, S28 = 92, S29 = 93, S30 = 94, S31 = 95, 
                //  96-103 -- OBSOLETE. F0-F7. Used by the FPA system. Superseded by VFP.
                // 104-111 -- wCGR0-wCGR7, ACC0-ACC7 (Intel wireless MMX)
                WR0 = 112,WR1 = 113,WR2 = 114,WR3 = 115,WR4 = 116,WR5 = 117,WR6 = 118,WR7 = 119,
                WR8 = 120,WR9 = 121,WR10 = 122,WR11 = 123,WR12 = 124,WR13 = 125,WR14 = 126,WR15 = 127,
                // 128-133 -- SPSR, SPSR_{FIQ|IRQ|ABT|UND|SVC}
                // 134-143 -- Reserved
                // 144-150 -- R8_USR-R14_USR
                // 151-157 -- R8_FIQ-R14_FIQ
                // 158-159 -- R13_IRQ-R14_IRQ
                // 160-161 -- R13_ABT-R14_ABT
                // 162-163 -- R13_UND-R14_UND
                // 164-165 -- R13_SVC-R14_SVC
                // 166-191 -- Reserved
                WC0 = 192,WC1 = 193,WC2 = 194,WC3 = 195,
                // 196-199 -- wC4-wC7 (Intel wireless MMX control)
                // 200-255 -- Reserved
                D0  = 256,D1  = 257,D2  = 258,D3  = 259,D4  = 260,D5  = 261,D6  = 262,D7  = 263,
                D8  = 264,D9  = 265,D10 = 266,D11 = 267,D12 = 268,D13 = 269,D14 = 270,D15 = 271,
                D16 = 272,D17 = 273,D18 = 274,D19 = 275,D20 = 276,D21 = 277,D22 = 278,D23 = 279,
                D24 = 280,D25 = 281,D26 = 282,D27 = 283,D28 = 284,D29 = 285,D30 = 286,D31 = 287,
                // 288-319 -- Reserved for VFP/Neon
                // 320-8191 -- Reserved
                // 8192-16383 -- Unspecified vendor co-processor register.
}
enum_as_consts!(ARM32Reg,
                UNW_ARM_R0  = R0,
                UNW_ARM_R1  = R1,
                UNW_ARM_R2  = R2,
                UNW_ARM_R3  = R3,
                UNW_ARM_R4  = R4,
                UNW_ARM_R5  = R5,
                UNW_ARM_R6  = R6,
                UNW_ARM_R7  = R7,
                UNW_ARM_R8  = R8,
                UNW_ARM_R9  = R9,
                UNW_ARM_R10 = R10,
                UNW_ARM_R11 = R11,
                UNW_ARM_R12 = R12,
                UNW_ARM_SP  = SP,  // Logical alias for UNW_REG_SP
                UNW_ARM_R13 = SP,
                UNW_ARM_LR  = LR,
                UNW_ARM_R14 = LR,
                UNW_ARM_IP  = IP,  // Logical alias for UNW_REG_IP
                UNW_ARM_R15 = IP,
                // 16-63 -- OBSOLETE. Used in VFP1 to represent both S0-S31 and D0-D31.
                UNW_ARM_S0  = S0,
                UNW_ARM_S1  = S1,
                UNW_ARM_S2  = S2,
                UNW_ARM_S3  = S3,
                UNW_ARM_S4  = S4,
                UNW_ARM_S5  = S5,
                UNW_ARM_S6  = S6,
                UNW_ARM_S7  = S7,
                UNW_ARM_S8  = S8,
                UNW_ARM_S9  = S9,
                UNW_ARM_S10 = S10,
                UNW_ARM_S11 = S11,
                UNW_ARM_S12 = S12,
                UNW_ARM_S13 = S13,
                UNW_ARM_S14 = S14,
                UNW_ARM_S15 = S15,
                UNW_ARM_S16 = S16,
                UNW_ARM_S17 = S17,
                UNW_ARM_S18 = S18,
                UNW_ARM_S19 = S19,
                UNW_ARM_S20 = S20,
                UNW_ARM_S21 = S21,
                UNW_ARM_S22 = S22,
                UNW_ARM_S23 = S23,
                UNW_ARM_S24 = S24,
                UNW_ARM_S25 = S25,
                UNW_ARM_S26 = S26,
                UNW_ARM_S27 = S27,
                UNW_ARM_S28 = S28,
                UNW_ARM_S29 = S29,
                UNW_ARM_S30 = S30,
                UNW_ARM_S31 = S31,
                //  96-103 -- OBSOLETE. F0-F7. Used by the FPA system. Superseded by VFP.
                // 104-111 -- wCGR0-wCGR7, ACC0-ACC7 (Intel wireless MMX)
                UNW_ARM_WR0 = WR0,
                UNW_ARM_WR1 = WR1,
                UNW_ARM_WR2 = WR2,
                UNW_ARM_WR3 = WR3,
                UNW_ARM_WR4 = WR4,
                UNW_ARM_WR5 = WR5,
                UNW_ARM_WR6 = WR6,
                UNW_ARM_WR7 = WR7,
                UNW_ARM_WR8 = WR8,
                UNW_ARM_WR9 = WR9,
                UNW_ARM_WR10 = WR10,
                UNW_ARM_WR11 = WR11,
                UNW_ARM_WR12 = WR12,
                UNW_ARM_WR13 = WR13,
                UNW_ARM_WR14 = WR14,
                UNW_ARM_WR15 = WR15,
                // 128-133 -- SPSR, SPSR_{FIQ|IRQ|ABT|UND|SVC}
                // 134-143 -- Reserved
                // 144-150 -- R8_USR-R14_USR
                // 151-157 -- R8_FIQ-R14_FIQ
                // 158-159 -- R13_IRQ-R14_IRQ
                // 160-161 -- R13_ABT-R14_ABT
                // 162-163 -- R13_UND-R14_UND
                // 164-165 -- R13_SVC-R14_SVC
                // 166-191 -- Reserved
                UNW_ARM_WC0 = WC0,
                UNW_ARM_WC1 = WC1,
                UNW_ARM_WC2 = WC2,
                UNW_ARM_WC3 = WC3,
                // 196-199 -- wC4-wC7 (Intel wireless MMX control)
                // 200-255 -- Reserved
                UNW_ARM_D0  = D0,
                UNW_ARM_D1  = D1,
                UNW_ARM_D2  = D2,
                UNW_ARM_D3  = D3,
                UNW_ARM_D4  = D4,
                UNW_ARM_D5  = D5,
                UNW_ARM_D6  = D6,
                UNW_ARM_D7  = D7,
                UNW_ARM_D8  = D8,
                UNW_ARM_D9  = D9,
                UNW_ARM_D10 = D10,
                UNW_ARM_D11 = D11,
                UNW_ARM_D12 = D12,
                UNW_ARM_D13 = D13,
                UNW_ARM_D14 = D14,
                UNW_ARM_D15 = D15,
                UNW_ARM_D16 = D16,
                UNW_ARM_D17 = D17,
                UNW_ARM_D18 = D18,
                UNW_ARM_D19 = D19,
                UNW_ARM_D20 = D20,
                UNW_ARM_D21 = D21,
                UNW_ARM_D22 = D22,
                UNW_ARM_D23 = D23,
                UNW_ARM_D24 = D24,
                UNW_ARM_D25 = D25,
                UNW_ARM_D26 = D26,
                UNW_ARM_D27 = D27,
                UNW_ARM_D28 = D28,
                UNW_ARM_D29 = D29,
                UNW_ARM_D30 = D30,
                UNW_ARM_D31 = D31
                // 288-319 -- Reserved for VFP/Neon
                // 320-8191 -- Reserved
                // 8192-16383 -- Unspecified vendor co-processor register.
                );

// OpenRISC1000 register numbers
enum OR1KReg {
    R0  = 0, R1  = 1, R2  = 2, R3  = 3, R4  = 4, R5  = 5, R6  = 6, R7  = 7, 
    R8  = 8, R9  = 9, R10 = 10,R11 = 11,R12 = 12,R13 = 13,R14 = 14,R15 = 15,
    R16 = 16,R17 = 17,R18 = 18,R19 = 19,R20 = 20,R21 = 21,R22 = 22,R23 = 23,
    R24 = 24,R25 = 25,R26 = 26,R27 = 27,R28 = 28,R29 = 29,R30 = 30,R31 = 31,
}
enum_as_consts!(OR1KReg, 
                UNW_OR1K_R0  = R0,
                UNW_OR1K_R1  = R1,
                UNW_OR1K_R2  = R2,
                UNW_OR1K_R3  = R3,
                UNW_OR1K_R4  = R4,
                UNW_OR1K_R5  = R5,
                UNW_OR1K_R6  = R6,
                UNW_OR1K_R7  = R7,
                UNW_OR1K_R8  = R8,
                UNW_OR1K_R9  = R9,
                UNW_OR1K_R10 = R10,
                UNW_OR1K_R11 = R11,
                UNW_OR1K_R12 = R12,
                UNW_OR1K_R13 = R13,
                UNW_OR1K_R14 = R14,
                UNW_OR1K_R15 = R15,
                UNW_OR1K_R16 = R16,
                UNW_OR1K_R17 = R17,
                UNW_OR1K_R18 = R18,
                UNW_OR1K_R19 = R19,
                UNW_OR1K_R20 = R20,
                UNW_OR1K_R21 = R21,
                UNW_OR1K_R22 = R22,
                UNW_OR1K_R23 = R23,
                UNW_OR1K_R24 = R24,
                UNW_OR1K_R25 = R25,
                UNW_OR1K_R26 = R26,
                UNW_OR1K_R27 = R27,
                UNW_OR1K_R28 = R28,
                UNW_OR1K_R29 = R29,
                UNW_OR1K_R30 = R30,
                UNW_OR1K_R31 = R31);

pub struct UnwindContext {
    repr: unw_context_t,
}

pub struct UnwindCursor {
    repr: unw_cursor_t,
}

impl UnwindContext {
    pub fn new() -> UnwindContext {
        unsafe {
            let mut c = UnwindContext { repr: mem::uninitialized() };
            if 0 != unw_getcontext(&mut c.repr as *mut _) {
                panic!("got error from unw_getcontext");
            }
            return c;
        }
    }
    pub fn cursor(&self) -> UnwindCursor {
        unsafe {
            let mut c = UnwindCursor { repr: mem::uninitialized() };
            if 0 != unw_init_local(&mut c.repr as *mut _, &self.repr as *const _) {
                panic!("got error from unw_init_local");
            }
            return c;
        }
    }
}

macro_rules! match_err {
    ($val:ident, $($Variant:ident),*; $default:expr) => {
        match $val {
            $(x if x == UnwindError::$Variant as c_int => UnwindError::$Variant),*,
            _ => $default,
        }
    }
}

macro_rules! match_errs {
    ($val:ident, $default:expr) => {
        match_err!($val, Unspec, NoMem, BadReg, ReadOnlyReg, StopUnwind,
                   InvalidIP, BadFrame, Invalid, BadVersion, NoInfo;
                   $default)
    }
}

impl UnwindCursor {
    fn ffi_ret<C>(&self, context: &'static str, c: C) -> Result<c_int, UnwindError>
        where C: FnOnce(*const unw_cursor_t) -> c_int
    {
        unsafe {
            match c(&self.repr as *const _) {
                x if x >= 0 => Ok(x),
                x => Err(match_errs!(x, panic!("got error from {}", context))),
            }
        }
    }

    fn ffi<T, C>(&self, context: &'static str, c: C) -> Result<T, UnwindError>
        where C: FnOnce(*const unw_cursor_t, *mut T) -> c_int
    {
        unsafe {
            let mut t: T = mem::uninitialized();
            match self.ffi_ret(context, |cursor| c(cursor, &mut t as *mut _)) {
                Ok(_) => Ok(t),
                Err(e) => { mem::forget(t); Err(e) }
            }
        }
    }
    
    pub fn step(mut self) -> Result<Option<UnwindCursor>, UnwindError> {
        self.ffi_ret("unw_step", |cursor| unsafe { unw_step(cursor as *mut _) })
            .map(|x| if x == 0 { None } else { Some(self) })
    }

    pub fn get_reg(&self, reg: unw_regnum_t) -> Result<unw_word_t, UnwindError> {
        self.ffi("unw_get_reg", |cursor, p_word| unsafe { unw_get_reg(cursor, reg, p_word) })
    }

    pub fn get_fpreg(&self, reg: unw_regnum_t) -> Result<unw_fpreg_t, UnwindError> {
        self.ffi("unw_get_fpreg", |cursor, p_fpreg| unsafe { unw_get_fpreg(cursor, reg, p_fpreg) })
    }

    pub fn resume(&self) -> Result<(), UnwindError> {
        self.ffi("unw_resume", |cursor, _p_unit| unsafe { unw_resume(cursor) })
    }

    pub fn get_proc_info(&self) -> Result<unw_proc_info_t, UnwindError> {
        //! Warning: I keep getting core dumps from within a call to
        //! strncp (within `unw_get_proc_info) when I try to use this
        //! function. Presumably it has some extra preconditions on
        //! p_pip (or cursor?) but I cannot figure out from the
        //! documentation what they might be.
        //!
        //! In any case, I believe I can extract the infomation I need
        //! via `fn get_proc_name` and `fn get_reg(IP)`
        self.ffi("unw_get_proc_info", |cursor, p_pip| unsafe { unw_get_proc_info(cursor, p_pip) })
    }

    // (This is apparently not offered by the LLVM libunwind fork?)
    #[cfg(not_now)]
    pub fn get_save_loc(&self, i: c_int) -> Result<unw_save_loc_t, UnwindError> {
        self.ffi("unw_get_save_loc", |cursor, p_loc| unsafe { unw_get_save_loc(cursor, i, p_loc) })
    }

    pub fn is_signal_frame(&self) -> Result<bool, UnwindError> {
        self.ffi_ret("unw_is_signal_frame", |cursor| unsafe { unw_is_signal_frame(cursor) })
            .map(|ret| ret > 0)
    }

    pub fn get_proc_name(&self, bufp: &mut Vec<u8>) -> Result<unw_word_t, UnwindError> {
        unsafe {
            let mut off: unw_word_t = mem::uninitialized();
            let r: *const unw_cursor_t = &self.repr;
            let p: *mut u8 = bufp.as_mut_ptr();
            let p: *mut c_char = p as *mut c_char;
            let c: size_t = bufp.capacity();
            let offp: *mut unw_word_t = &mut off as *mut unw_word_t;
            // println!("calling unw_get_proc_name r: {:?} p: {:?} c: {:?} offp", r, p, c);
            let ret: c_int = unw_get_proc_name(r, p, c, offp);
            if ret != 0 {
                return Err(match_errs!(ret, panic!("got error from unw_get_proc_name")));
            }
            let len = libc::strlen(p);
            bufp.set_len(len);
            Ok(off)
        }
    }

}

pub fn is_fpreg(regnum: unw_regnum_t) -> bool {
    unsafe {
        0 != unw_is_fpreg(regnum)
    }
}

pub fn regname(regnum: unw_regnum_t) -> &'static CStr {
    unsafe {
        CStr::from_ptr(unw_regname(regnum))
    }
}
