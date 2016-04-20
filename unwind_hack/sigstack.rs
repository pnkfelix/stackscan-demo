use libc::{c_int, c_void, size_t};

pub struct stack_t {
    ss_sp: *mut c_void,
    ss_flags: c_int,
    ss_size: size_t,
}
