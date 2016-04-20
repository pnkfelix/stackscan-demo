use libc::{c_ulong};
use std::mem;

// const _SIGSET_NWORDS: usize = 1024 / (8 * mem::size_of::<c_ulong>());

#[cfg(any(target_os = "windows",
          target_os = "android"))]
const SIZEOF_ULONG: usize = 4;

#[cfg(target_os = "linux")]
#[cfg(any(target_arch="mips",
          target_arch="x86",
          target_arch="arm",
          target_arch="powerpc"))]
const SIZEOF_ULONG: usize = 4;

#[cfg(any(solaris))]
const SIZEOF_ULONG: usize = 8;

#[cfg(target_os = "linux")]
#[cfg(any(target_arch="x86_64",
          target_arch="aarch64",
          target_arch="powerpc64"))]
const SIZEOF_ULONG: usize = 8;
           
const _SIGSET_NWORDS: usize = 1024 / (8 * SIZEOF_ULONG);

pub struct __sigset_t {
    __val: [c_ulong; _SIGSET_NWORDS],
}
