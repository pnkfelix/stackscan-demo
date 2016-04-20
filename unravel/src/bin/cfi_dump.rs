use std::io::Cursor;
use std::env;
use std::path::Path;

extern crate elf;
extern crate unravel;

pub fn main() {
    if let Some(arg1) = env::args().nth(1) {
        let path = Path::new(&arg1);
        let file = match elf::File::open_path(&path) {
            Ok(f) => f,
            Err(e) => panic!("Error: {:?}", e),
        };

        let eh_frame = match file.get_section(".eh_frame") {
            Some(s) => s,
            None => panic!("Failed to look up .eh_frame section"),
        };


        println!("Found .eh_frame section, length = {} bytes",
                 eh_frame.data.len());

        let mut cursor = Cursor::new(eh_frame.data.as_slice());
        loop {
            match unravel::dwarf::eh_frame::read_cfi_entry(&mut cursor) {
                Ok(cfi_entry) => {
                    println!("Found CFI entry:");
                    println!("{}", cfi_entry);
                }
                Err(err) => panic!("Failed to read CFI entry: {}", err),
            }
        }
    } else {
        println!("Usage: {} <path>", env::args().nth(0).unwrap())
    }
}
