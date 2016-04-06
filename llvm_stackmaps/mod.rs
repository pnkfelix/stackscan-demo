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
        Truncated { want: &'static str, at: usize },
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

macro_rules! impl_to_usize {
    ($($S:ident),*) => {$(
        impl ToUsize for $S {
            fn to_usize(&self) -> usize { self.0 as usize }
        }
        )*}
}

impl_to_usize!(NumFunctions, NumConstants, NumRecords, NumLocations, NumLiveOuts);

impl StackMap {
    pub fn read_from<T>(r: &[u8]) -> Result<Self, ReadError>
        where T:ByteOrder
    {
        ReadFrom::<T>::read(r, 0).map(|x|x.0)
    }
}

type ReadResult<X> = Result<(X, usize), ReadError>;

trait ReadFrom<T:ByteOrder>: Sized {
    fn read(bytes: &[u8], offset: usize) -> ReadResult<Self>;
}

trait ReadMany<T:ByteOrder>: ReadFrom<T> {
    type Count: ToUsize;
    fn read_many(bytes: &[u8],
                 mut offset: usize,
                 count: &Self::Count) -> ReadResult<Vec<Self>> {
        let mut v = Vec::with_capacity(count.to_usize());
        for _ in 0..(count.to_usize()) {
            let (e, new_offset) = Self::read(bytes, offset)?;
            offset = new_offset;
            v.push(e);
        }
        Ok((v, offset))
    }
}

impl<T:ByteOrder> ReadFrom<T> for StackMap {
    fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
        let (hr, i): (Header, _) = ReadFrom::<T>::read(b, i)?;
        let (nf, i): (NumFunctions, _) = ReadFrom::<T>::read(b, i)?;
        let (nc, i): (NumConstants, _) = ReadFrom::<T>::read(b, i)?;
        let (nr, i): (NumRecords, _) = ReadFrom::<T>::read(b, i)?;
        let (ss, i): (Vec<StackSize>, _) = ReadMany::<T>::read_many(b, i, &nf)?;
        let (cs, i): (Vec<LargeConstant>, _) = ReadMany::<T>::read_many(b, i, &nc)?;
        let (rs, i): (Vec<Record>, _) = ReadMany::<T>::read_many(b, i, &nr)?;
        Ok((StackMap { header: hr,
                       stack_sizes: ss,
                       large_constants: cs,
                       records: rs }, i))
    }
}

impl<T:ByteOrder> ReadFrom<T> for Header {
    fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
        let (version, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        let (reserved_1, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        let (reserved_2, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        Ok((Header { version: version,
                     reserved_1: reserved_1,
                     reserved_2: reserved_2, }, i))
    }
}

macro_rules! impl_read_for_prim {
    ($t:ident, $read:ident) => {
        impl<T:ByteOrder> ReadFrom<T> for $t {
            fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
                let size = ::std::mem::size_of::<$t>();
                if b.len() < (i + size) {
                    return Err(PrimError::Truncated {
                        want: stringify!($t),
                        at: i,
                    }.into());
                }
                Ok((T::$read(b.split_at(i).1), i+size))
            }
        }
    }
}

impl_read_for_prim!(u16, read_u16);
impl_read_for_prim!(u32, read_u32);
impl_read_for_prim!(i32, read_i32);
impl_read_for_prim!(u64, read_u64);

impl<T:ByteOrder> ReadFrom<T> for u8 {
    fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
        if b.len() < (i + 1) {
            return Err(PrimError::Truncated {
                want: stringify!(u8),
                at: i,
            }.into());
        }
        Ok((b[i], i+1))
    }
}

macro_rules! impl_read_from_for_u32_wrapper {
    ($S:ident) => {
        impl<T:ByteOrder> ReadFrom<T> for $S {
            fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
                if b.len() < (i + 4) {
                    return Err(PrimError::Truncated {
                        want: stringify!($S),
                        at: i,
                    }.into());
                }
                Ok(($S(T::read_u32(b.split_at(i).1)), 4))
            }
        }
    }
}

impl_read_from_for_u32_wrapper!(NumFunctions);
impl_read_from_for_u32_wrapper!(NumConstants);
impl_read_from_for_u32_wrapper!(NumRecords);
impl_read_from_for_u32_wrapper!(NumLocations);
impl_read_from_for_u32_wrapper!(NumLiveOuts);

impl<T:ByteOrder> ReadMany<T> for StackSize { type Count = NumFunctions; }
impl<T:ByteOrder> ReadMany<T> for LargeConstant { type Count = NumConstants; }
impl<T:ByteOrder> ReadMany<T> for Record { type Count = NumRecords; }
impl<T:ByteOrder> ReadMany<T> for Location { type Count = NumLocations; }

impl<T:ByteOrder> ReadFrom<T> for StackSize {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        let (fa, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        let (ss, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        Ok((StackSize { function_address: fa,
                        stack_size: ss }, i))
    }
}

impl<T:ByteOrder> ReadFrom<T> for LargeConstant {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        let (value, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        Ok((LargeConstant { value: value }, i))
    }
}

impl<T:ByteOrder> ReadFrom<T> for Record {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        let (pi, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        let (io, i): (u32, _) = ReadFrom::<T>::read(b, i)?;
        let (rs, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        let (nl, i): (NumLocations, _) = ReadFrom::<T>::read(b, i)?;
        let (ls, i): (Vec<Location>, _) = ReadMany::<T>::read_many(b, i, &nl)?;
        let (p1, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        let (nlo, i): (NumLiveOuts, _) = ReadFrom::<T>::read(b, i)?;
        let (los, i): (Vec<LiveOut>, _) = ReadMany::<T>::read_many(b, i, &nlo)?;
        assert!(i % 4 == 0);
        let (p2, i) = if i % 8 == 0 {
            (None, i) } else {
            let (p2, i): (u32, _) = ReadFrom::<T>::read(b, i)?;
            (Some(p2), i)
        };
        Ok((Record {
            patchpoint_id: pi,
            instruction_offset: io,
            reserved: rs,
            num_locations: nl,
            locations: ls,
            padding_1: p1,
            num_live_outs: nlo,
            live_outs: los,
            padding_2: p2,
        }, i))
    }
}

impl<T:ByteOrder> ReadFrom<T> for Location {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        let (vt, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        let (rs, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        let (dr, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        let (os, i): (i32, _) = ReadFrom::<T>::read(b, i)?;
        Ok((Location {
            val_type: vt,
            reserved: rs,
            dwarf_regnum: dr,
            offset_or_small_constant: os,
        }, i))
    }    
}

impl<T:ByteOrder> ReadMany<T> for LiveOut { type Count = NumLiveOuts; }

impl<T:ByteOrder> ReadFrom<T> for LiveOut {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        let (dr, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        let (rs, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        let (sb, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        Ok((LiveOut {
            dwarf_regnum: dr,
            reserved: rs,
            size_in_bytes: sb, }, i))
    }
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
