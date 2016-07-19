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

use std::fmt;
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
    pub enum LocationError {
        Prim(PrimError),
        InvalidVariant(u8),
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
        Location(LocationError),
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
pub struct NumLocations(pub u16);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumLiveOuts(pub u16);

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
        debug!("StackMap::read b: {:?} i: {}", b, i);
        let (hr, i): (Header, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackMap::read hr: {:?} i: {}", hr, i);
        let (nf, i): (NumFunctions, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackMap::read nf: {:?} i: {}", nf, i);
        let (nc, i): (NumConstants, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackMap::read nc: {:?} i: {}", nc, i);
        let (nr, i): (NumRecords, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackMap::read nr: {:?} i: {}", nr, i);
        let (ss, i): (Vec<StackSize>, _) = ReadMany::<T>::read_many(b, i, &nf)?;
        debug!("StackMap::read ss: {:?} i: {}", ss, i);
        let (cs, i): (Vec<LargeConstant>, _) = ReadMany::<T>::read_many(b, i, &nc)?;
        debug!("StackMap::read cs: {:?} i: {}", cs, i);
        let (rs, i): (Vec<Record>, _) = ReadMany::<T>::read_many(b, i, &nr)?;
        debug!("StackMap::read rs: {:?} i: {}", rs, i);
        Ok((StackMap { header: hr,
                       stack_sizes: ss,
                       large_constants: cs,
                       records: rs }, i))
    }
}

impl<T:ByteOrder> ReadFrom<T> for Header {
    fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
        debug!("Header::read b: {:?} i: {}", b, i);
        let (version, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Header::read version: {} i: {}", version, i);
        let (reserved_1, i): (u8, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Header::read reserved_1: {} i: {}", reserved_1, i);
        let (reserved_2, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Header::read reserved_2: {} i: {}", reserved_2, i);
        Ok((Header { version: version,
                     reserved_1: reserved_1,
                     reserved_2: reserved_2, }, i))
    }
}

macro_rules! impl_read_for_prim {
    ($t:ident, $read:ident, $debug:expr) => {
        impl<T:ByteOrder> ReadFrom<T> for $t {
            fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
                let size = ::std::mem::size_of::<$t>();
                if b.len() < (i + size) {
                    return Err(PrimError::Truncated {
                        want: stringify!($t),
                        at: i,
                    }.into());
                }
                let ret = Ok((T::$read(b.split_at(i).1), i+size));
                if $debug {
                    println!("{}::read({:?}, {}) => {:?}",
                             stringify!($t), b, i, ret);
                }
                ret
            }
        }
    }
}

impl_read_for_prim!(u16, read_u16, false);
impl_read_for_prim!(u32, read_u32, false);
impl_read_for_prim!(i32, read_i32, false);
impl_read_for_prim!(u64, read_u64, true);

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

macro_rules! impl_read_from_for_u_wrapper {
    ($S:ident, $read_u:ident, $size:expr) => {
        impl<T:ByteOrder> ReadFrom<T> for $S {
            fn read(b: &[u8], i: usize) -> Result<(Self, usize), ReadError> {
                if b.len() < (i + $size) {
                    return Err(PrimError::Truncated {
                        want: stringify!($S),
                        at: i,
                    }.into());
                }
                Ok(($S(T::$read_u(b.split_at(i).1)), i+$size))
            }
        }
    }
}

macro_rules! impl_read_from_for_u32_wrapper {
    ($S:ident) => { impl_read_from_for_u_wrapper!($S, read_u32, 4); }
}

macro_rules! impl_read_from_for_u16_wrapper {
    ($S:ident) => { impl_read_from_for_u_wrapper!($S, read_u16, 2); }
}

impl_read_from_for_u32_wrapper!(NumFunctions);
impl_read_from_for_u32_wrapper!(NumConstants);
impl_read_from_for_u32_wrapper!(NumRecords);

impl_read_from_for_u16_wrapper!(NumLocations);
impl_read_from_for_u16_wrapper!(NumLiveOuts);

impl<T:ByteOrder> ReadMany<T> for StackSize { type Count = NumFunctions; }
impl<T:ByteOrder> ReadMany<T> for LargeConstant { type Count = NumConstants; }
impl<T:ByteOrder> ReadMany<T> for Record { type Count = NumRecords; }
impl<T:ByteOrder> ReadMany<T> for Location { type Count = NumLocations; }

impl<T:ByteOrder> ReadFrom<T> for StackSize {
    fn read(b: &[u8], i: usize) -> ReadResult<Self> {
        debug!("StackSize::read b: {:?} i: {}", b, i);
        let (fa, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackSize::read fa: {:?} i: {}", fa, i);
        let (ss, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        debug!("StackSize::read ss: {:?} i: {}", ss, i);
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
        debug!("Record::read b: {:?} i: {}", b, i);
        let (pi, i): (u64, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read pi: {:?}, i: {}", pi, i);
        let (io, i): (u32, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read io: {:?}, i: {}", io, i);
        let (rs, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read rs: {:?}, i: {}", rs, i);
        let (nl, i): (NumLocations, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read nl: {:?}, i: {}", nl, i);
        let (ls, i): (Vec<Location>, _) = ReadMany::<T>::read_many(b, i, &nl)?;
        debug!("Record::read ls: {:?}, i: {}", ls, i);
        let (p1, i): (u16, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read p1: {:?}, i: {}", p1, i);
        let (nlo, i): (NumLiveOuts, _) = ReadFrom::<T>::read(b, i)?;
        debug!("Record::read nlo: {:?}, i: {}", nlo, i);
        let (los, i): (Vec<LiveOut>, _) = ReadMany::<T>::read_many(b, i, &nlo)?;
        debug!("Record::read los: {:?}, i: {}", los, i);
        assert!(i % 4 == 0);
        let (p2, i) = if i % 8 == 0 {
            (None, i) } else {
            let (p2, i): (u32, _) = ReadFrom::<T>::read(b, i)?;
            (Some(p2), i)
        };
        debug!("Record::read p2: {:?}, i: {}", p2, i);
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

        let variant = match vt {
            0x1 => LocationVariant::Register { dwarf_regnum: dr },
            0x2 => LocationVariant::Direct { dwarf_regnum: dr, offset: os },
            0x3 => LocationVariant::Indirect { dwarf_regnum: dr, offset: os },
            0x4 => LocationVariant::Constant { value : os },
            0x5 => LocationVariant::ConstIndex { offset: os },
            other => return Err(LocationError::InvalidVariant(other).into()),
        };

        Ok((Location {
            reserved: rs,
            variant: variant,
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

#[derive(Clone, PartialEq, Eq)]
pub struct StackSize {
    function_address: u64,
    stack_size: u64,
}

impl fmt::Debug for StackSize {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "StackSize {{ function_address 0x{:x}, stack_size: {} }}",
               self.function_address,
               self.stack_size)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LargeConstant {
    pub value: u64,
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
    reserved: u8,
    variant: LocationVariant,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LocationVariant {
    Register { dwarf_regnum: u16 }, // (offset_or_small_constant 0 in this case)
    Direct   { dwarf_regnum: u16, offset: i32 },
    Indirect { dwarf_regnum: u16, offset: i32 },
    Constant { value: i32 },
    ConstIndex { offset: i32 }, // I *assume* the dwarf_regnum is unused here...
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

impl StackMap {
    pub fn header(&self) -> &Header { &self.header }
    pub fn stack_sizes(&self) -> &[StackSize] { &self.stack_sizes[..] }
    pub fn large_constants(&self) -> &[LargeConstant] { &self.large_constants[..] }
    pub fn records(&self) -> &[Record] { &self.records[..] }
}

impl Header {
    pub fn version(&self) -> u8 { self.version }
}

impl StackSize {
    pub fn function_address(&self) -> u64 { self.function_address }
    pub fn stack_size(&self) -> u64 { self.stack_size }
}

impl Record {
    pub fn patchpoint_id(&self) -> u64 { self.patchpoint_id }
    pub fn instruction_offset(&self) -> u32 { self.instruction_offset }
    pub fn locations(&self) -> &[Location] { &self.locations[..] }
    pub fn live_outs(&self) -> &[LiveOut] { &self.live_outs[..] }
}

impl Location {
    pub fn variant(&self) -> &LocationVariant { &self.variant }
}

impl LiveOut {
    pub fn regnum(&self) -> u16 { self.dwarf_regnum }
    pub fn size_in_bytes(&self) -> u8 { self.size_in_bytes }
}
