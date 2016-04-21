use libc::{c_char, c_double, c_int, c_void, size_t};
use std::mem;
use std::ffi::CStr;

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
pub type unw_regnum_t = c_int;

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

#[link(name="unwind")]
#[link(name="unwind-x86_64")]
extern {
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_getcontext"]
    fn unw_getcontext(ucp: *mut unw_context_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_init_local"]
    fn unw_init_local(c: *mut unw_cursor_t, ctxt: *const unw_context_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_step"]
    fn unw_step(cp: *mut unw_cursor_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_get_reg"]
    fn unw_get_reg(cp: *const unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_get_fpreg"]
    fn unw_get_fpreg(cp: *const unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_fpreg_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_resume"]
    fn unw_resume(cp: *const unw_cursor_t) -> c_int;

    // I think the methods (and global) below are solely used for "remote
    // unwinding", which is not immediately relevant to me, so I need
    // not worry about finding definition for `struct unw_addr_space`.
    
    // static unw_local_addr_space: unw_addr_space_t;
    // fn unw_init_remote(c: *mut unw_cursor_t, as: unw_addr_space_t, arg: *mut c_void);
    // fn unw_create_addr_space(ap: *mut unw_accessors_t, byteorder: c_int);
    // fn unw_destroy_addr_space(as: unw_addr_space_t);
    // fn unw_get_accessors(as: unw_addr_space_t) -> *mut unw_accessors_t;
    // fn unw_flush_cache(as: unw_addr_space, lo: unw_word_t, hi: unw_word_t);
    // fn unw_set_caching_policy(as: unw_addr_space, policy: unw_caching_policy_t) -> c_int;

    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_regname"]
    fn unw_regname(regnum: unw_regnum_t) -> *const c_char;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_get_proc_name"]
    fn unw_get_proc_info(cp: *const unw_cursor_t, pip: *mut unw_proc_info_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_get_save_loc"]
    fn unw_get_save_loc(cp: *const unw_cursor_t, _int: c_int, loc: *mut unw_save_loc_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_is_fpreg"]
    fn unw_is_fpreg(r: unw_regnum_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_is_signal_frame"]
    fn unw_is_signal_frame(cp: *const unw_cursor_t) -> c_int;
    #[cfg(target_arch="x86_64")]
    #[link_name="_Ux86_64_get_proc_name"]
    fn unw_get_proc_name(cp: *const unw_cursor_t, bufp: *mut c_char, len: size_t, offp: *mut unw_word_t) -> c_int;

    // fn _U_dyn_register(di: *mut unw_dyn_info_t);
    // fn _U_dyn_cancel(di: *mut unw_dyn_info_t);
}

pub struct UnwindContext {
    repr: unw_context_t,
}

pub struct UnwindCursor {
    repr: unw_cursor_t,
}

#[repr(i32)]
#[derive(Debug)]
pub enum UnwindError {
    /// Unspecified (general) error
    Unspec = -1,
    /// Out of memory
    NoMem = -2,
    /// Bad register number
    BadReg = -3,
    /// Attempt to write read-only register
    ReadOnlyReg = -4,
    /// Stop Unwinding
    StopUnwind = -5,
    /// Invalid IP
    InvalidIP = -6,
    /// Bad Frame
    BadFrame = -9,
    /// Unsupported Operation or Bad Value
    Invalid = -10,
    /// Unwind Info has unsupported version
    BadVersion = -11,
    /// No Unwind Info found
    NoInfo = -12,
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
    pub fn step(mut self) -> Result<Option<UnwindCursor>, UnwindError> {
        unsafe {
            let ret = unw_step(&mut self.repr as *mut _);
            match ret {
                0 => Ok(None),
                x if x > 0 => Ok(Some(self)),
                x => Err(match_errs!(x, panic!("got error from unw_step")))
            }
        }
    }

    pub fn get_reg(&self, reg: unw_regnum_t) -> Result<unw_word_t, UnwindError> {
        unsafe {
            let mut w: unw_word_t = mem::uninitialized();
            let ret = unw_get_reg(&self.repr as *const _, reg, &mut w as *mut _);
            match ret {
                0 => Ok(w),
                x => Err(match_errs!(x, panic!("got error from unw_get_reg"))),
            }
        }
    }

    pub fn get_fpreg(&self, reg: unw_regnum_t) -> Result<unw_fpreg_t, UnwindError> {
        unsafe {
            let mut w: unw_fpreg_t = mem::uninitialized();
            let ret = unw_get_fpreg(&self.repr as *const _, reg, &mut w as *mut _);
            match ret {
                0 => Ok(w),
                x => Err(match_errs!(x, panic!("got error from unw_get_fpreg"))),
            }
        }
    }

    pub fn resume(&self) -> Result<(), UnwindError> {
        unsafe {
            let ret = unw_resume(&self.repr as *const _);
            match ret {
                0 => Ok(()),
                x => Err(match_errs!(x, panic!("got error from unw_resume"))),
            }
        }
    }

    pub fn get_proc_info(&self) -> Result<unw_proc_info_t, UnwindError> {
        unsafe {
            let mut pip: unw_proc_info_t = mem::uninitialized();
            let ret = unw_get_proc_info(&self.repr as *const _, &mut pip as *mut _);
            match ret {
                0 => Ok(pip),
                x => Err(match_errs!(x, panic!("got error from get_proc_info"))),
            }
        }
    }

    pub fn get_save_loc(&self, i: c_int) -> Result<unw_save_loc_t, UnwindError> {
        unsafe {
            let mut loc: unw_save_loc_t = mem::uninitialized();
            let ret = unw_get_save_loc(&self.repr as *const _, i, &mut loc as *mut _);
            match ret {
                0 => Ok(loc),
                x => Err(match_errs!(x, panic!("got error from unw_get_save_loc"))),
            }
        }
    }

    pub fn is_signal_frame(&self) -> Result<bool, UnwindError> {
        unsafe {
            let ret = unw_is_signal_frame(&self.repr as *const _);
            match ret {
                0 => Ok(false),
                x if x > 0 => Ok(true),
                x => Err(match_errs!(x, panic!("got error from unw_is_signal_frame"))),
            }
        }
    }

    pub fn get_proc_name(&self, bufp: &mut Vec<c_char>) -> Result<unw_word_t, UnwindError> {
        unsafe {
            let mut off: unw_word_t = mem::uninitialized();
            let r: *const unw_cursor_t = &self.repr;
            let p: *mut c_char = bufp.as_mut_ptr();
            let c: size_t = bufp.capacity();
            let offp: *mut unw_word_t = &mut off as *mut unw_word_t;

            // The line below seems to cause an LLVM assert fail on my branch.

            // let ret: c_int = unw_get_proc_name(r, p, c, offp);
            // match ret {
            //     0 => Ok(off),
            //     x => Err(match_errs!(x, panic!("got error from unw_get_proc_name"))),
            // }
            unimplemented!()
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
