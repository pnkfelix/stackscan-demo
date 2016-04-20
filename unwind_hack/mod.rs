use libc::{c_char, c_double, c_int, size_t};

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

extern {
    fn unw_getcontext(ucp: *mut unw_context_t);
    fn unw_init_local(c: *mut unw_cursor_t, ctxt: *mut unw_context_t);
    // fn unw_init_remote(c: *mut unw_cursor_t, as: unw_addr_space_t, arg: *mut c_void);
    fn unw_step(cp: *mut unw_cursor_t);
    fn unw_get_reg(cp: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t);
    fn unw_get_fpreg(cp: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_fpreg_t);
    fn unw_resume(cp: *mut unw_cursor_t);

    // fn unw_create_addr_space(ap: *mut unw_accessors_t, byteorder: c_int);
    // fn unw_destroy_addr_space(as: unw_addr_space_t);
    // fn unw_get_accessors(as: unw_addr_space_t) -> *mut unw_accessors_t;
    // fn unw_flush_cache(as: unw_addr_space, lo: unw_word_t, hi: unw_word_t);
    // fn unw_set_caching_policy(as: unw_addr_space, policy: unw_caching_policy_t) -> c_int;

    fn unw_regname(regnum: unw_regnum_t) -> *const c_char;
    // fn unw_get_proc_info(cp: *mut unw_cursor_t, pip: *mut unw_proc_info_t);
    // fn unw_get_save_loc(cp: *mut unw_cursor_t, _int: c_int, loc: *mut unw_save_loc_t);
    fn unw_is_fpreg(r: unw_regnum_t) -> c_int;
    fn unw_is_signal_frame(cp: *mut unw_cursor_t) -> c_int;
    fn unw_get_proc_name(cp: *mut unw_cursor_t, bufp: *mut c_char, len: size_t, offp: *mut unw_word_t) -> c_int;

    // fn _U_dyn_register(di: *mut unw_dyn_info_t);
    // fn _U_dyn_cancel(di: *mut unw_dyn_info_t);
}
