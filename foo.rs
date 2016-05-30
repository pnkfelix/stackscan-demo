#![feature(core_intrinsics, patchpoint_call_intrinsic, stackmap_call_intrinsic)]
#![feature(question_mark)]
// #![feature(type_ascription)]
// #![feature(rustc_attrs)]
#![feature(libc)]
#![feature(const_fn)]
#![feature(rustc_private)]
#![allow(unused_features)]

extern crate libc;
#[macro_use] extern crate log;

extern crate util;

pub use util::*;

use std::intrinsics;


const DEMO_ID: i64 = 0;
const SUBCALL1_ID: i64 = 1;
const SUBCALL2_ID: i64 = 2;

#[no_mangle]
pub extern "C" fn subcall_1(data: *mut u8) {
    println!("Enter `subcall_1`");
    unsafe {
        intrinsics::stackmap_call(SUBCALL1_ID, 0, subcall_2, data);
        // intrinsics::patchpoint_call(SUBCALL1_ID, 13, subcall_2, data);
    }
    println!("Finis `subcall_1`");
}

#[no_mangle]
pub extern "C" fn subcall_2(data: *mut u8) {
    println!("Enter `subcall_2`");
    let mut i = 0;
    let mut saw_one = false;
    print!("[");
    let mut datum = unsafe { *data.offset(i) as char };
    while datum != '\0' {
        if saw_one {
            print!(", ");
        }
        print!("{}", datum);
        saw_one = true;
        i += 1;
        datum = unsafe { *data.offset(i) as char };
    }
    println!("]");
    unsafe {
        intrinsics::stackmap_call(SUBCALL2_ID, 0, subcall_3, data);
        // intrinsics::patchpoint_call(SUBCALL2_ID, 13, subcall_3, data);
        
    }
    println!("Finis `subcall_2`");
}

use self::backtrace_hack::ReturnAddress;
use self::backtrace_hack::dlinfo_fbase;

#[derive(Debug)]
enum FrameInfo<'a> {
    Addr(ReturnAddress),
    StackMap(StackMapFrameInfo<'a>),
}

#[derive(Debug)]
struct StackMapFrameInfo<'a> {
    addr: ReturnAddress,
    patchpoint_id: u64,
    instruction_offset: u32,
    locations: &'a [Location],
    live_outs: &'a [LiveOut],
}

trait StackMapExt {
    fn frame_info(&self, addr: ReturnAddress) -> FrameInfo;
}

impl StackMapExt for StackMap {
    fn frame_info(&self, addr: ReturnAddress) -> FrameInfo {
        let mut frame_info = None;

        debug!("");
        debug!("addr: {:?}", addr);
        
	if let Some(_base) = dlinfo_fbase(addr.0) {
            // Okay, we have a base address for the shared object of addr

            // We still need to find the function itself as well as
            // the call-site within it; here's a naive way to do it.
            //
            // FIXME: build structure ahead of time that allows
            // something faster than this linear scan.
            for r in self.records() {
                let fn_address = address_id(r.patchpoint_id()).unwrap();
                let instr_offset = r.instruction_offset() as usize;
                let call_address = fn_address + instr_offset;

                debug!("frame_info addr {:?} \
                                     record {:?} \
                                     fn_address: 0x{:x} \
                                     instr_offset: {:?} \
                                     call_address: 0x{:x} \
                                     delta: {}",
                         addr, r, fn_address, instr_offset, call_address,
                         call_address as isize - (addr.0 as isize));
                
                if call_address == addr.0 as usize {
                    assert!(frame_info.is_none());
                    frame_info = Some(StackMapFrameInfo {
                        addr: addr,
                        patchpoint_id: r.patchpoint_id(),
                        instruction_offset: r.instruction_offset(),
                        locations: r.locations(),
                        live_outs: r.live_outs(),
                    })
                }
            }
        } else {
            println!("frame_info no fbase for {:?}", addr);
        }

        if let Some(frame_info) = frame_info {
            FrameInfo::StackMap(frame_info)
        } else {
            FrameInfo::Addr(addr)
        }
    }
}

#[no_mangle]
pub extern "C" fn subcall_3(_data: *mut u8) {
    println!("Start `subcall_3`");

    let map = unsafe { STACK_MAP.get() };
    println!("map:                              {:?}", map);

    let unw = unwind_hack::UnwindContext::new();
    let mut cursor = Some(unw.cursor());

    while let Some(c) = cursor {
        let ip = c.get_reg(unwind_hack::UNW_REG_IP).unwrap();
        let sp = c.get_reg(unwind_hack::UNW_REG_SP).unwrap();
        println!("ip: 0x{:x} sp: 0x{:x}", ip, sp);
        let opt_fbase = backtrace_hack::dlinfo_fbase(ip as *const libc::c_void);
        println!("dlinfo_fbase: {:?}", opt_fbase);
        if let Some(fbase) = opt_fbase {
            println!("dlrel ip: 0x{:x}", ip - (fbase as u64));
        }

        let (offset, name) = {
            let mut buf = Vec::with_capacity(256);
            match c.get_proc_name(&mut buf) {
                Err(e) => {
                    println!("failed to .get_proc_name from subcall_3: {:?}", e);
                    cursor = c.step().unwrap();
                    println!("");
                    continue;
                }
                Ok(offset) => (offset, String::from_utf8(buf).unwrap()),
            }
        };
        println!("ip: 0x{:08x} name: {} offset: {}", ip, name, offset);
        println!("start of fn:  0x{:08x}", ip - offset);
        if let Some(fbase) = opt_fbase {
            println!("rel start of fn:  0x{:08x}", ip - offset - (fbase as u64));
        }
        cursor = c.step().unwrap();
        println!("");
    }
    println!("Finis `subcall_3`");
}

#[cfg(use_backtrace_hack)]
fn subcall_3(data: *mut u8) {
    println!("Start `subcall_3`");
    let map = unsafe { STACK_MAP.get() };
    println!("map:                   {:?}", map);
    let addresses = backtrace_hack::backtrace_return_addresses();
    println!("backtrace:             {:?}", addresses);
    let dlinfos = backtrace_hack::backtrace_dlinfos(&addresses);
    println!("backtrace dlinfos:     {:?}", dlinfos);
    let frame_infos: Vec<_> = addresses.iter()
        .map(|a| map.frame_info(*a))
        .collect();
    println!("backtrace frame_infos: {:?}", frame_infos);
    println!("Finis `subcall_3`");
}

fn main() {
    let local: u32 = 0;
    unsafe { MAIN_LOCAL = &local as *const _ as usize; }
    demo().unwrap();
}

#[derive(Debug)]
pub enum DemoError {
    ParseError(elf::ParseError),
    MissingSection(String),
}

impl std::convert::From<elf::ParseError> for DemoError {
    fn from(x: elf::ParseError) -> Self { DemoError::ParseError(x) }
}

use self::llvm_stackmaps::{LiveOut, Location, StackMap};
use std::cell::UnsafeCell;

struct SharedStackMap {
    map: UnsafeCell<Option<StackMap>>
}

impl SharedStackMap {
    /// Initializes the map. Unsafe because it does not attempt to
    /// enforce thread safety in any way: its your own job to make
    /// sure you do this once and only once.
    unsafe fn initialize(&self, map: StackMap) { *self.map.get() = Some(map); }
    const fn new() -> Self { SharedStackMap { map: UnsafeCell::new(None) } }
    unsafe fn get(&self) -> &'static StackMap {
        (*self.map.get()).as_ref().expect("Cannot get an uninitialized stack map")
    }
}

/// I am going to manually ensure that all clients of SharedStackMap access
/// it in solely a read-only fashion.
unsafe impl Sync for SharedStackMap { }

static STACK_MAP: SharedStackMap = SharedStackMap::new();
static mut ADDRESS_IDS: [usize; 3] = [0; 3];
fn address_id(patchpoint_id: u64) -> Option<usize> {
    unsafe {
        if patchpoint_id > ::std::usize::MAX as u64 { return None; }
        let idx = patchpoint_id as usize;
        if idx >= ADDRESS_IDS.len() { return None; }
        Some(ADDRESS_IDS[idx])
    }
}

fn initialize_shared_state() -> Result<(), DemoError> {
    use std::path::Path;
    use self::byteorder::LittleEndian;

    // FIXME: LLVM's stackmap/patchpoint intrinsic API does
    // not embed the identity of the function context where the
    // stackmap/patchpoint.
    //
    // It instead requires each stackmap/patchpoint invocation to
    // include a (globally unique) 64-bit ID, known at compile time.
    //
    // The problem for me is that it means that I have the
    // responsibility of building a mapping from functions to their
    // unique ID's, just so I can map from the return address to the
    // appropriate stackmap record. :(
    //
    // (I do not quite understand why they made this so hard;
    // presumably it is because the stackmap/patchpoint interface was
    // designed for use in JITs, not in AOT compilers.)
    //
    // In any case, for now I can just hardcode a table with the
    // necessary mapping from patchpoint IDs to function addresses.
    // (And hopefully I can hack the Rust backend to generate a table
    // like this automatically, though ensuring the IDs are unique
    // will be a bit harder than what I do here. I guess for that
    // I might as well re-use the SVH approach... we'll cross that
    // bridge when we get to it.)
    unsafe {
        ADDRESS_IDS[DEMO_ID as usize] = demo as usize;
        ADDRESS_IDS[SUBCALL1_ID as usize] = subcall_1 as usize;
        ADDRESS_IDS[SUBCALL2_ID as usize] = subcall_2 as usize;
    }

    // FIXME: this is not a reliable way to extract the executable file path.
    // (look into alternatives)
    let binary = std::env::args().next().unwrap();
    
    println!("Hello World from {}", binary);

    let path = Path::new(&binary);
    let file = elf::File::open_path(&path)?;

    let stackmap_section = {
        let name = ".llvm_stackmaps".to_string();
        file.get_section(name.clone())
            .ok_or_else(|| DemoError::MissingSection(name))?
    };

    println!("stackmap_section: {:?}", stackmap_section);

    let stack_map = StackMap::read_from::<LittleEndian>(&mut &stackmap_section.data[..]);
    println!("stack_map: {:?}", stack_map);
    let map = stack_map.expect("Cannot do demo without valid stack map");
    unsafe { STACK_MAP.initialize(map); }
    Ok(())
}

#[no_mangle]
// #[rustc_mir(borrowck_graphviz_postflow="/tmp/foo.dot")]
pub fn demo() -> Result<(), DemoError> {
    try!(initialize_shared_state());
    
    let mut data = vec![b'h', b'e', b'l', b'l', b'o', 0];
    println!("data addr: {:?}", &data as *const _);

    unsafe {
        intrinsics::stackmap_call(DEMO_ID, 0, subcall_1, data.as_mut_ptr());
        // intrinsics::patchpoint_call(DEMO_ID, 13, subcall_1, data.as_mut_ptr());
    }
    
    println!("Goodbye World");

    Ok(())
}
