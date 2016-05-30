#![feature(question_mark)]
#![feature(libc)]
#![feature(rustc_private)]

#![crate_type="lib"]
#![crate_name="util"]

extern crate libc;
#[macro_use] extern crate log;

#[allow(dead_code)]
#[path="byteorder/mod.rs"]
pub mod byteorder;

#[allow(dead_code, unused_imports, unused_variables, unused_mut)]
#[path="elf/mod.rs"]
pub mod elf;

#[path="backtrace_hack/mod.rs"]
pub mod backtrace_hack;

#[path="llvm_stackmaps/mod.rs"]
pub mod llvm_stackmaps;

#[path="unwind_hack/mod.rs"]
pub mod unwind_hack;

#[path="unravel/src/lib.rs"]
pub mod unravel;

pub static mut MAIN_LOCAL: usize = 0;
pub fn main_height() -> usize { unsafe { MAIN_LOCAL } }
