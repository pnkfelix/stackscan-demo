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

type unw_context_t = ucontext::ucontext_t;

extern {
    fn unw_getcontext(ucp: *mut unw_context_t);
}
