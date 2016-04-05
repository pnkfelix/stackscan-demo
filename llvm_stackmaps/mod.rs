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

pub trait ReadFrom: Sized {
    type Error;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error>;
}

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

pub trait OrderedReadFrom: ByteOrder + Sized {
    fn read_from<R:Read, S:ReadFrom>(r: &mut R) -> Result<S, S::Error> {
        S::read_from::<R, Self>(r)
    }
}

impl OrderedReadFrom for BigEndian { }
impl OrderedReadFrom for LittleEndian { }

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

macro_rules! impl_read_from_constant_u32 {
    ($This:ident, $name:expr) => {
        impl ReadFrom for $This {
            type Error = ConstantError;
            fn read_from<R,T>(r: &mut R) -> Result<Self, Self::Error>
                where R:Read, T:OrderedReadFrom
            {
                Ok($This(match r.read_u32::<T>() {
                    Ok(num) => num,
                    Err(err) => return Err(ConstantError { name: $name, err: err }),
                }))
            }
        }
        impl ToUsize for $This { fn to_usize(&self) -> usize { self.0 as usize } }
    }
}

impl_read_from_constant_u32!(NumFunctions, "num_functions");
impl_read_from_constant_u32!(NumConstants, "num_constants");
impl_read_from_constant_u32!(NumRecords, "num_records");
impl_read_from_constant_u32!(NumLocations, "num_locations");
impl_read_from_constant_u32!(NumLiveOuts, "num_live_outs");

trait ReadMany<T:OrderedReadFrom>: Sized {
    type Count: ToUsize;
    fn read<R:Read>(r: &mut R, c: &Self::Count) -> Result<Vec<Self>, ReadError>;
}

fn read_many<RM, R, T>(r: &mut R, c: &RM::Count) -> Result<Vec<RM>, ReadError>
    where RM: ReadFrom + ReadMany<T>, R:Read, T:OrderedReadFrom, ReadError: From<RM::Error>
{
        let mut v = Vec::with_capacity(c.to_usize());
        for _ in 0..(c.to_usize()) {
            v.push(T::read_from(r)?: RM)
        }
        Ok(v)
}

macro_rules! impl_read_many {
    ($Struct:ident, $Count:ident) => {
        impl<T:OrderedReadFrom> ReadMany<T> for $Struct {
            type Count = $Count;
            fn read<R:Read>(r: &mut R, c: &Self::Count) -> Result<Vec<Self>, ReadError> {
                read_many::<Self, R, T>(r, c)
            }
        }
    }
}

impl_read_many!(StackSize, NumFunctions);
impl_read_many!(LargeConstant, NumConstants);
impl_read_many!(Record, NumRecords);
impl_read_many!(Location, NumLocations);
impl_read_many!(LiveOut, NumLiveOuts);

impl StackMap {
    pub fn read_from<T>(mut r: &[u8]) -> Result<Self, ReadError>
        where T:OrderedReadFrom
    {
        ReadFrom::read_from::<&[u8],T>(&mut r)
    }
}

impl ReadFrom for StackMap {
    type Error = ReadError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let header: Header = T::read_from(r)?;
        let num_functions: NumFunctions = T::read_from(r)?;
        let num_constants: NumConstants = T::read_from(r)?;
        let num_records: NumRecords = T::read_from(r)?;
        let stack_size_records: Vec<StackSize> = ReadMany::<T>::read(r, &num_functions)?;
        let large_constants: Vec<LargeConstant> = ReadMany::<T>::read(r, &num_constants)?;
        let records: Vec<Record> = ReadMany::<T>::read(r, &num_records)?;
        Ok(StackMap {
            header: header,
            stack_sizes: stack_size_records,
            large_constants: large_constants,
            records: records,
        })
    }
}

impl ReadFrom for Header {
    type Error = HeaderError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let version: u8 = T::read_from(r)?;
        let reserved_1: u8 = T::read_from(r)?;
        let reserved_2: u16 = T::read_from(r)?;
        Ok(Header { version: version,
                    reserved_1: reserved_1,
                    reserved_2: reserved_2 })
    }
}

impl ReadFrom for StackSize {
    type Error = StackSizeError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let function_address: u64 = T::read_from(r)?;
        let stack_size: u64 = T::read_from(r)?;
        Ok(StackSize { function_address: function_address,
                       stack_size: stack_size })
    }
}

impl ReadFrom for LargeConstant {
    type Error = LargeConstantError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let value: u64 = T::read_from(r)?;
        Ok(LargeConstant { value: value })
    }
}

impl ReadFrom for Record {
    type Error = ReadError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let patchpoint_id: u64 = T::read_from(r)?;
        let instruction_offset: u32 = T::read_from(r)?;
        let reserved_1: u16 = T::read_from(r)?;
        let num_locations: NumLocations = T::read_from(r)?;
        let locations: Vec<Location> = ReadMany::<T>::read(r, &num_locations)?;
        let padding_1:u16 = T::read_from(r)?;
        let num_live_outs: NumLiveOuts = T::read_from(r)?;
        let live_outs: Vec<LiveOut> = ReadMany::<T>::read(r, &num_live_outs)?;
        Ok(Record {
            patchpoint_id: patchpoint_id,
            instruction_offset: instruction_offset,
            reserved: reserved_1,
            num_locations: num_locations,
            locations: locations,
            padding_1: padding_1,
            num_live_outs: num_live_outs,
            live_outs: live_outs,
            padding_2: None, // FIXME
        })
    }
}

impl ReadFrom for Location {
    type Error = ReadError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let val_type: u8 = T::read_from(r)?;
        let reserved: u8 = T::read_from(r)?;
        let dwarf_regnum: u16 = T::read_from(r)?;
        let offset_or_small_constant: i32 = T::read_from(r)?;
        Ok(Location {
            val_type: val_type,
            reserved: reserved,
            dwarf_regnum: dwarf_regnum,
            offset_or_small_constant: offset_or_small_constant,
        })
    }
}

impl ReadFrom for LiveOut {
    type Error = ReadError;
    fn read_from<R:Read, T:OrderedReadFrom>(r: &mut R) -> Result<Self, Self::Error> {
        let dwarf_regnum: u16 = T::read_from(r)?;
        let reserved: u8 = T::read_from(r)?;
        let size_in_bytes: u8 = T::read_from(r)?;
        Ok(LiveOut {
            dwarf_regnum: dwarf_regnum,
            reserved: reserved,
            size_in_bytes: size_in_bytes,
        })
    }
}

macro_rules! impl_read_from_for_prim {
    (u8) => {
        impl ReadFrom for u8 {
            type Error = PrimError;
            fn read_from<R,T>(r: &mut R) -> Result<Self, Self::Error>
                where R:Read, T:OrderedReadFrom
            {
                Ok(r.read_u8()?)
            }
        }
    };
    ($prim:ident, $read:ident) => {
        impl ReadFrom for $prim {
            type Error = PrimError;
            fn read_from<R,T>(r: &mut R) -> Result<Self, Self::Error>
                where R:Read, T:OrderedReadFrom
            {
                Ok(r.$read::<T>()?)
            }
        }
    }
}

impl_read_from_for_prim!(u8);
impl_read_from_for_prim!(u16, read_u16);
impl_read_from_for_prim!(u32, read_u32);
impl_read_from_for_prim!(u64, read_u64);

impl_read_from_for_prim!(i32, read_i32);

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
