use libc::{c_char, c_double, c_int, c_void, size_t};

#[cfg(target_arch="x86_64")]
#[path="ucontext_x86_64.rs"]
mod ucontext;

// This almost certainly needs to be made specific to just x86 (and
// add cases for the other processors).
#[cfg(not(target_arch="x86_64"))]
#[path="ucontext_generic.rs"]
mod ucontext;

// These almost certainly need to be replaced with something that is
// target/platform specific.
mod sigset;
mod sigstack;

#[cfg(target_arch="x86_64")]
const UNW_TDEP_CURSOR_LEN: usize = 127;

#[cfg(target_arch="x86_64")]
type unw_word_t = usize;

#[repr(C)]
pub struct unw_cursor_t {
    opaque: [unw_word_t; UNW_TDEP_CURSOR_LEN],
}

type unw_context_t = ucontext::ucontext_t;
type unw_regnum_t = c_int;

#[cfg(target_arch="x86_64")]
type unw_fpreg_t = c_double;

#[cfg(target_arch="x86_64")]
#[repr(C)]
struct unw_tdep_proc_info_t {

}

#[repr(C)]
struct unw_proc_info_t {
    start_ip: unw_word_t,
    end_ip: unw_word_t,
    lsda: unw_word_t,
    handler: unw_word_t,
    gp: unw_word_t,
    flags: unw_word_t,
    format: c_int,
    unwind_info_size: c_int,
    unwind_info: *mut c_void,
    extra: unw_tdep_proc_info_t,
}

#[repr(C)]
enum unw_save_loc_type_t {
    UNW_SLT_NONE,
    UNW_SLT_MEMORY,
    UNW_SLT_REG,
}

#[repr(C)]
struct unw_save_loc_t {
    type_: unw_save_loc_type_t,
    u: unw_word_t, // should be union { unw_word_t, unw_regnum_t }
    extra: unw_tdep_save_loc_t,
}

#[cfg(target_arch="x86_64")]
#[repr(C)]
struct unw_tdep_save_loc_t {

}

extern {
    fn unw_getcontext(ucp: *mut unw_context_t);
    fn unw_init_local(c: *mut unw_cursor_t, ctxt: *mut unw_context_t);
    fn unw_step(cp: *mut unw_cursor_t);
    fn unw_get_reg(cp: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t);
    fn unw_get_fpreg(cp: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_fpreg_t);
    fn unw_resume(cp: *mut unw_cursor_t);

    // I think the methods below are solely used for "remote
    // unwinding", which is not immediately relevant to me, so I need
    // not worry about finding definition for `struct unw_addr_space`.
    
    // fn unw_init_remote(c: *mut unw_cursor_t, as: unw_addr_space_t, arg: *mut c_void);
    // fn unw_create_addr_space(ap: *mut unw_accessors_t, byteorder: c_int);
    // fn unw_destroy_addr_space(as: unw_addr_space_t);
    // fn unw_get_accessors(as: unw_addr_space_t) -> *mut unw_accessors_t;
    // fn unw_flush_cache(as: unw_addr_space, lo: unw_word_t, hi: unw_word_t);
    // fn unw_set_caching_policy(as: unw_addr_space, policy: unw_caching_policy_t) -> c_int;

    fn unw_regname(regnum: unw_regnum_t) -> *const c_char;
    fn unw_get_proc_info(cp: *mut unw_cursor_t, pip: *mut unw_proc_info_t);
    fn unw_get_save_loc(cp: *mut unw_cursor_t, _int: c_int, loc: *mut unw_save_loc_t);
    fn unw_is_fpreg(r: unw_regnum_t) -> c_int;
    fn unw_is_signal_frame(cp: *mut unw_cursor_t) -> c_int;
    fn unw_get_proc_name(cp: *mut unw_cursor_t, bufp: *mut c_char, len: size_t, offp: *mut unw_word_t) -> c_int;

    // fn _U_dyn_register(di: *mut unw_dyn_info_t);
    // fn _U_dyn_cancel(di: *mut unw_dyn_info_t);
}
