use libc::{c_char, c_int, c_void};
use std::mem;
use std::ptr;

extern {
    fn backtrace(buffer: *mut *mut c_void, size: c_int) -> c_int;
    fn backtrace_symbols(buffer: *mut *const c_void, size: c_int) -> *mut *mut c_char;
    fn backtrace_symbols_fd(buffer: *mut *const c_void, size: c_int, fd: c_int);
}

pub fn stack_height() -> usize {
    let local: u32 = 0;
    let height = &local as *const _ as usize;
    super::main_height() - height
}

pub fn backtrace_return_addresses() -> Vec<*const c_void> {
    let height = stack_height();
    let word_size = mem::size_of::<usize>();
    let rounded_height = (height + (word_size - 1)) / word_size;
    assert!(rounded_height >= 0);
    assert!(rounded_height <= ::std::i32::MAX as usize);
    let mut buffer = vec![ptr::null(); rounded_height];
    let rounded_height = rounded_height as i32;
    unsafe {
        let filled_size = backtrace(buffer.as_mut_ptr() as *mut *mut c_void,
                                    rounded_height);
        assert!(filled_size >= 0);
        buffer.set_len(filled_size as usize);
    }
    buffer
}
