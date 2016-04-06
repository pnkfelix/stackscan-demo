//! The LLVM Stack Map Section format is documented here:
//!
//! http://llvm.org/docs/StackMaps.html#stack-map-format
//!
//! portions of which I am transcribing here.
//!
//! ```
//! Header {
//!     uint8  : Stack Map Version (current version is 1)
//!     uint8  : Reserved (expected to be 0)
//!     uint16 : Reserved (expected to be 0)
//! }
//! uint32 : NumFunctions
//! uint32 : NumConstants
//! uint32 : NumRecords
//! StkSizeRecord[NumFunctions] {
//!     uint64 : Function Address
//!     uint64 : Stack Size
//! }
//! Constants[NumConstants] {
//!     uint64 : LargeConstant
//! }
//! StkMapRecord[NumRecords] {
//!     uint64 : PatchPoint ID
//!     uint32 : Instruction Offset
//!     uint16 : Reserved (record flags)
//!     uint16 : NumLocations
//!     Location[NumLocations] {
//!         uint8  : Register | Direct | Indirect | Constant | ConstantIndex
//!         uint8  : Reserved (location flags)
//!         uint16 : Dwarf RegNum
//!         int32  : Offset or SmallConstant
//!     }
//!     uint16 : Padding
//!     uint16 : NumLiveOuts
//!     LiveOuts[NumLiveOuts] {
//!         uint16 : Dwarf RegNum
//!         uint8  : Reserved
//!         uint8  : Size in Bytes
//!     }
//!     uint32 : Padding (only if required to align to 8 byte)
//! }
//! ```

use byteorder::{ByteOrder, ReadBytesExt};
use byteorder::{BigEndian, LittleEndian};



use std::io::{Read};

pub trait Many {
    type Count;
}

pub mod errors {
    use std::io;

    #[derive(Debug)]
    pub enum StackMapError {
        Truncated { reading: &'static str },
        Header(HeaderError),
    }

    #[derive(Debug)]
    pub enum HeaderError {
        Prim(PrimError),
    }
    #[derive(Debug)]
    pub struct ConstantError { pub name: &'static str, pub err: io::Error }
    #[derive(Debug)]
    pub enum StackSizeError {
        Prim(PrimError),
    }
    #[derive(Debug)]
    pub enum LargeConstantError {
        Prim(PrimError),
    }
    #[derive(Debug)]
    pub enum RecordError {
        Prim(PrimError),
        Constant(ConstantError),
    }
    #[derive(Debug)]
    pub enum PrimError {
        Io(io::Error),
    }

    macro_rules! impl_from_for_read_error {
        ($VariantErr:ident, $Variant:ident) => {
            impl From<$VariantErr> for ReadError {
                fn from(x: $VariantErr) -> Self { ReadError::$Variant(x) }
            }
        }
    }

    macro_rules! enum_read_error {
        {$($Variant:ident($VariantErr:ident),)*} => {
            #[derive(Debug)]
            pub enum ReadError {
                $($Variant($VariantErr),)*
            }
            $(impl_from_for_read_error!{$VariantErr, $Variant})*
        }
    }

    enum_read_error!{
        StackMap(StackMapError),
        Constant(ConstantError),
        Header(HeaderError),
        StackSize(StackSizeError),
        LargeConstant(LargeConstantError),
        Record(RecordError),
        Prim(PrimError),
    }
    
    impl From<io::Error> for PrimError {
        fn from(x: io::Error) -> Self { PrimError::Io(x) }
    }

    impl From<PrimError> for HeaderError {
        fn from(x: PrimError) -> Self { HeaderError::Prim(x) }
    }

    impl From<PrimError> for StackSizeError {
        fn from(x: PrimError) -> Self { StackSizeError::Prim(x) }
    }

    impl From<PrimError> for LargeConstantError {
        fn from(x: PrimError) -> Self { LargeConstantError::Prim(x) }
    }

    impl From<PrimError> for RecordError {
        fn from(x: PrimError) -> Self { RecordError::Prim(x) }
    }

    impl From<ConstantError> for RecordError {
        fn from(x: ConstantError) -> Self { RecordError::Constant(x) }
    }
}

use self::errors::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumFunctions(pub u32);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumConstants(pub u32);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumRecords(pub u32);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumLocations(pub u32);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumLiveOuts(pub u32);

trait ToUsize { fn to_usize(&self) -> usize; }

impl StackMap {
    pub fn read_from<T>(r: &[u8]) -> Result<Self, ReadError>
        where T:ByteOrder
    {
        ReadFrom<T>::read_from(r)
    }
}

trait ReadFrom<T:ByteOrder> {
    //
    fn read_from(bytes: &[u8]) -> Option<(Self, usize)>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackMap {
    header: Header,
    stack_sizes: Vec<StackSize>,
    large_constants: Vec<LargeConstant>,
    records: Vec<Record>
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Header {
    version: u8,
    reserved_1: u8,
    reserved_2: u16,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackSize {
    function_address: u64,
    stack_size: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LargeConstant {
    value: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Record {
    patchpoint_id: u64,
    instruction_offset: u32,
    reserved: u16,
    num_locations: NumLocations,
    locations: Vec<Location>,
    padding_1: u16,
    num_live_outs: NumLiveOuts,
    live_outs: Vec<LiveOut>,
    padding_2: Option<u32>, // only if required to align to 8 byte
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Location {
    val_type: u8,
    reserved: u8,
    dwarf_regnum: u16,
    offset_or_small_constant: i32,
}

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ValType { Register, Direct, Indirect, Constant, ConstantIndex }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LiveOut {
    dwarf_regnum: u16,
    reserved: u8,
    size_in_bytes: u8,
}
