use libc::{c_char, c_void};
use std::ffi::CStr;
use std::mem;
use std::ptr;

mod bt {
    use libc::{c_char, c_int, c_void};
    extern {
        pub fn backtrace(buffer: *mut *mut c_void,
                         size: c_int) -> c_int;
        pub fn backtrace_symbols(buffer: *mut *const c_void,
                                 size: c_int) -> *mut *mut c_char;
        #[cfg(not_now)]
        pub fn backtrace_symbols_fd(buffer: *mut *const c_void,
                                    size: c_int,
                                    fd: c_int);
    }
}

mod dl {
    use libc::{c_char, c_int, c_void};
    #[derive(Debug)]
    #[repr(C)]
    pub struct Dl_info {
        pub dli_fname: *const c_char,
        pub dli_fbase: *mut c_void,
        pub dli_sname: *const c_char,
        pub dli_saddr: *mut c_void,
    }

    extern {
        pub fn dladdr(addr: *mut c_void,
                      info: *mut Dl_info) -> c_int;
        #[cfg(not_now)]
        pub fn dladdr1(addr: *mut c_void,
                       info: *mut Dl_info,
                       extra_info: *mut *mut c_void,
                       flags: c_int) -> c_int;
    }

    #[cfg(not_now)]
    const RTLD_DL_LINKMAP: i32 = 2;
    #[cfg(not_now)]
    const RTLD_DL_SYMENT: i32 = 1;

    pub trait RequestFlags {
        fn flags(&self) -> c_int;
    }

    #[cfg(not_now)]
    impl RequestFlags for LinkMap {
        fn flags(&self) -> c_int { RTLD_DL_LINKMAP as c_int }
    }
    #[cfg(not_now)]
    impl RequestFlags for SymEnt {
        fn flags(&self) -> c_int { RTLD_DL_SYMENT as c_int }
    }

    #[cfg(not_now)]
    #[repr(C)]
    pub struct LinkMap {
        l_addr: libc::size_t,
        l_name: *const char,
        l_ld: ElfDyn,
        l_next: *mut LinkMap,
        l_prev: *mut LinkMap,
    }

    #[cfg(not_now)]
    #[repr(C)]
    #[cfg(target_pointer_width = "32")]
    pub struct SymEnt {
        /// Symbol name (starting tbl index)
        st_name: u32,
        /// Symbol value
        st_value: u32,
        /// Symbol size
        st_size: u32,
        /// Symbol type and binding
        st_info: c_uchar,
        /// Symbol visibility
        st_other: c_uchar,
        /// Section index
        st_shndx: u16,
    }

    #[cfg(not_now)]
    #[repr(C)]
    #[cfg(target_pointer_width = "64")]
    pub struct SymEnt {
        /// Symbol name (starting tbl index)
        st_name: u64,
        /// Symbol type and binding
        st_info: c_uchar,
        /// Symbol visibility
        st_other: c_uchar,
        /// Section index
        st_shndx: u16,
        /// Symbol value
        st_value: u32,
        /// Symbol size
        st_size: u32,
    }

    #[cfg(target_pointer_width = "32")]
    #[repr(C)]
    pub struct ElfDyn {
        d_tag: i32,
        d_un: u32,
    }

    #[cfg(not_now)]
    #[cfg(target_pointer_width = "64")]
    #[repr(C)]
    pub struct ElfDyn {
        d_tag: i64,
        d_un: u64,
    }
}

pub fn stack_height() -> usize {
    let local: u32 = 0;
    let height = &local as *const _ as usize;
    super::main_height() - height
}

#[derive(Copy, Clone, Debug)]
pub struct ReturnAddress(pub *const c_void);

pub fn backtrace_return_addresses() -> Vec<ReturnAddress> {
    let height = stack_height();
    let word_size = mem::size_of::<usize>();
    let rounded_height = (height + (word_size - 1)) / word_size;
    assert!(rounded_height <= ::std::i32::MAX as usize);
    let mut buffer: Vec<*const c_void> = vec![ptr::null(); rounded_height];
    let rounded_height = rounded_height as i32;
    unsafe {
        let filled_size = bt::backtrace(mem::transmute(buffer.as_mut_ptr()),
                                        rounded_height);
        assert!(filled_size >= 0);
        buffer.set_len(filled_size as usize);
        mem::transmute(buffer)
    }
}

pub fn backtrace_symbols(addresses: &Vec<ReturnAddress>) -> Vec<&'static CStr> {
    let mut symbols = Vec::with_capacity(addresses.len());

    unsafe {
        let strings = bt::backtrace_symbols(mem::transmute(addresses.as_ptr()),
                                            addresses.len() as i32);
        for i in 0..addresses.len() {
            let c_str = CStr::from_ptr(*strings.offset(i as isize));
            symbols.push(c_str);
        }
    }

    symbols
}

#[derive(Debug)]
pub struct DlInfo {
    fname: String,
    fbase: *mut c_void,
    sname: String,
    saddr: *mut c_void,
}

/// If `addr` is an address is located in a currently-loaded shared
/// object, returns the base address at which that shared object is
/// located. Otherwise returns `None`.
pub fn dlinfo_fbase(addr: *const c_void) -> Option<*const c_void> {
    unsafe { 
        let mut info: dl::Dl_info = mem::uninitialized();
        let result = dl::dladdr(addr as *mut _, &mut info as *mut _);
        if 0 != result {
            Some(info.dli_fbase)
        } else {
            None
        }
    }
}

impl DlInfo {
    fn from_dl<'a>(info: &'a dl::Dl_info) -> DlInfo {
        fn to_string(p: *const c_char) -> String {
            if p.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string()
            }
        }
        let fname = to_string(info.dli_fname);
        let fbase = info.dli_fbase;
        let sname = to_string(info.dli_sname);
        let saddr = info.dli_saddr;
        DlInfo {
            fname: fname, fbase: fbase, sname: sname, saddr: saddr
        }
    }
}

#[derive(Debug)]
pub struct DlInfoUnmatched(*const c_void);

pub type DlInfoResult = Result<Vec<DlInfo>, (Vec<DlInfo>, Vec<DlInfoUnmatched>)>;

pub fn backtrace_dlinfos(addresses: &[ReturnAddress]) -> DlInfoResult {
    let (ok, err): (Vec<_>, Vec<_>) = addresses.iter()
        .map(|&ReturnAddress(addr)| {
            let mut info: dl::Dl_info = unsafe { mem::uninitialized() };
            let result = unsafe { dl::dladdr(addr as *mut _, &mut info as *mut _) };
            if 0 != result {
                Ok(DlInfo::from_dl(&info))
            } else {
                Err(DlInfoUnmatched(addr))
            }})
        .partition(|x|x.is_ok());

    let okay: Vec<DlInfo> = ok.into_iter().map(|r| r.unwrap()).collect();
    let errs: Vec<DlInfoUnmatched> = err.into_iter().map(|r| r.unwrap_err()).collect();
    if errs.len() == 0 {
        Ok(okay)
    } else {
        Err((okay, errs))
    }
}
