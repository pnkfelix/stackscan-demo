#![feature(core_intrinsics, patchpoint_call_intrinsic, stackmap_call_intrinsic)]
#![feature(question_mark)]
#![feature(type_ascription)]
#![allow(unused_features)]

use std::intrinsics;

#[allow(dead_code)]
mod byteorder;
#[allow(dead_code, unused_imports, unused_variables, unused_mut)]
mod elf;

mod llvm_stackmaps;

fn subcall(data: *mut u8) {
    println!("Enter `subcall`");
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
    println!("Finis `subcall`");
}

fn main() {
    demo().unwrap();
}

#[derive(Debug)]
enum DemoError {
    ParseError(elf::ParseError),
    MissingSection(String),
}

impl std::convert::From<elf::ParseError> for DemoError {
    fn from(x: elf::ParseError) -> Self { DemoError::ParseError(x) }
}

fn demo() -> Result<(), DemoError> {
    use std::path::Path;
    use self::llvm_stackmaps::StackMap;
    use self::byteorder::LittleEndian;

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
    
    let data = &mut [b'h', b'e', b'l', b'l', b'o', 0];
    unsafe {
        intrinsics::stackmap_call(0, 13, subcall, data.as_mut_ptr());
    }
    
    println!("Goodbye World");

    Ok(())
}
