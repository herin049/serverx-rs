use std::{
    fmt::{Debug, Display, Formatter},
    io,
    io::{Read, Seek},
};

use crate::{tag::TagType, NamedTag, Tag};

pub trait ResourceTracker {
    fn alloc(&mut self, count: usize) -> Result<(), NbtDecodeErr>;
    fn dealloc(&mut self, count: usize);
    fn allocated(&self) -> usize;
    fn wind(&mut self) -> Result<(), NbtDecodeErr>;
    fn unwind(&mut self);
    fn depth(&self) -> usize;
}

pub struct BasicResourceTracker {
    pub allocated: usize,
    pub max_allocated: usize,
    pub depth: usize,
    pub max_depth: usize,
}

impl BasicResourceTracker {
    pub fn new(max_allocated: usize, max_depth: usize) -> Self {
        Self {
            allocated: 0,
            max_allocated,
            depth: 0,
            max_depth,
        }
    }
}

impl ResourceTracker for BasicResourceTracker {
    fn alloc(&mut self, count: usize) -> Result<(), NbtDecodeErr> {
        if self.allocated + count > self.max_allocated {
            Err(NbtDecodeErr::MemoryLimitExceeded(self.max_allocated))
        } else {
            self.allocated += count;
            Ok(())
        }
    }

    fn dealloc(&mut self, count: usize) {
        self.allocated -= count;
    }

    fn allocated(&self) -> usize {
        self.allocated
    }

    fn wind(&mut self) -> Result<(), NbtDecodeErr> {
        if self.depth >= self.max_depth {
            Err(NbtDecodeErr::RecursionLimitExceeded(self.max_depth))
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    fn unwind(&mut self) {
        self.depth -= 1;
    }

    fn depth(&self) -> usize {
        self.depth
    }
}

pub enum NbtDecodeErr {
    IoErr(io::Error),
    InvalidSeqLen(usize),
    UnexpectedTagType(TagType),
    InvalidTagType(i8),
    MemoryLimitExceeded(usize),
    RecursionLimitExceeded(usize),
    MalformedString,
}

impl Display for NbtDecodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtDecodeErr::IoErr(err) => write!(f, "io error: {}", err),
            NbtDecodeErr::InvalidSeqLen(len) => write!(f, "invalid sequence length : {}", len),
            NbtDecodeErr::UnexpectedTagType(tag) => write!(f, "unexpected tag type: {}", tag),
            NbtDecodeErr::InvalidTagType(tag) => write!(f, "invalid tag type with id: {}", tag),
            NbtDecodeErr::MemoryLimitExceeded(limit) => {
                write!(f, "memory limit of {} bytes exceeded", limit)
            }
            NbtDecodeErr::RecursionLimitExceeded(limit) => {
                write!(f, "recursion limit of {} bytes exceeded", limit)
            }
            NbtDecodeErr::MalformedString => write!(f, "malformed string"),
        }
    }
}

impl Debug for NbtDecodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

pub trait NbtDecode {
    type Repr;

    fn decode<R: Read + Seek, Re: ResourceTracker>(
        tag_type: TagType,
        reader: &mut R,
        tracker: &mut Re,
    ) -> Result<Self::Repr, NbtDecodeErr>;
}

macro_rules! number_impl {
    ($($num_ty:ty),*) => {
        $(
            impl NbtDecode for $num_ty {
                type Repr = Self;

                fn decode<R: Read + Seek, Re: ResourceTracker>(_tag_type: TagType, reader: &mut R, tracker: &mut Re) -> Result<Self::Repr, NbtDecodeErr> {
                    let mut buf = [0; std::mem::size_of::<Self>()];
                    reader.read_exact(buf.as_mut_slice()).map_err(|err| NbtDecodeErr::IoErr(err))?;
                    Ok(Self::from_be_bytes(buf))
                }
            }
        )*
    }
}

number_impl! { i8, i16, i32, i64, f32, f64 }

impl NbtDecode for String {
    type Repr = String;

    fn decode<R: Read + Seek, Re: ResourceTracker>(
        _tag_type: TagType,
        reader: &mut R,
        tracker: &mut Re,
    ) -> Result<Self::Repr, NbtDecodeErr> {
        let mut buf = [0; std::mem::size_of::<u16>()];
        reader
            .read_exact(buf.as_mut_slice())
            .map_err(|err| NbtDecodeErr::IoErr(err))?;
        let len = usize::from(u16::from_be_bytes(buf));
        tracker.alloc(len * std::mem::size_of::<u8>())?;
        let mut str_buf = vec![0u8; len];
        reader
            .read_exact(str_buf.as_mut_slice())
            .map_err(|err| NbtDecodeErr::IoErr(err))?;
        let str = String::from_utf8(str_buf).map_err(|_| NbtDecodeErr::MalformedString)?;
        Ok(str)
    }
}

impl NbtDecode for Tag {
    type Repr = Tag;

    fn decode<R: Read + Seek, Re: ResourceTracker>(
        tag_type: TagType,
        reader: &mut R,
        tracker: &mut Re,
    ) -> Result<Self::Repr, NbtDecodeErr> {
        match tag_type {
            TagType::End => Ok(Tag::End),
            TagType::Byte => Ok(Tag::Byte(<i8 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::Short => Ok(Tag::Short(<i16 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::Int => Ok(Tag::Int(<i32 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::Long => Ok(Tag::Long(<i64 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::Float => Ok(Tag::Float(<f32 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::Double => Ok(Tag::Double(<f64 as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::ByteArray => {
                let len =
                    usize::try_from(<i32 as NbtDecode>::decode(TagType::Int, reader, tracker)?)
                        .unwrap_or(0);
                tracker.alloc(len * std::mem::size_of::<u8>())?;
                let mut result = Vec::with_capacity(len);
                reader
                    .read_exact(result.as_mut_slice())
                    .map_err(|err| NbtDecodeErr::IoErr(err))?;
                Ok(Tag::ByteArray(result))
            }
            TagType::String => Ok(Tag::String(<String as NbtDecode>::decode(
                tag_type, reader, tracker,
            )?)),
            TagType::List => {
                let list_type_id = <i8 as NbtDecode>::decode(TagType::Byte, reader, tracker)?;
                let list_type = TagType::try_from(list_type_id)
                    .map_err(|err| NbtDecodeErr::InvalidTagType(list_type_id))?;
                let len =
                    usize::try_from(<i32 as NbtDecode>::decode(TagType::Int, reader, tracker)?)
                        .unwrap_or(0);
                tracker.alloc(len * std::mem::size_of::<Tag>())?;
                tracker.wind()?;
                let mut result = Vec::with_capacity(len);
                for _ in 0..len {
                    result.push(Tag::decode(list_type, reader, tracker)?);
                }
                tracker.unwind();
                Ok(Tag::List(result))
            }
            TagType::Compound => {
                let mut result = Vec::new();
                tracker.wind()?;
                loop {
                    let tag_type_id = <i8 as NbtDecode>::decode(TagType::Byte, reader, tracker)?;
                    let tag_type = TagType::try_from(tag_type_id)
                        .map_err(|err| NbtDecodeErr::InvalidTagType(tag_type_id))?;
                    if tag_type == TagType::End {
                        break;
                    }
                    tracker.alloc(std::mem::size_of::<NamedTag>())?;
                    result.push(NamedTag::decode(tag_type, reader, tracker)?);
                }
                tracker.unwind();
                Ok(Tag::Compound(result))
            }
            TagType::IntArray => {
                let len =
                    usize::try_from(<i32 as NbtDecode>::decode(TagType::Int, reader, tracker)?)
                        .unwrap_or(0);
                tracker.alloc(len * std::mem::size_of::<i32>())?;
                let mut result = Vec::with_capacity(len);
                for _ in 0..len {
                    result.push(<i32 as NbtDecode>::decode(TagType::Int, reader, tracker)?);
                }
                Ok(Tag::IntArray(result))
            }
            TagType::LongArray => {
                let len =
                    usize::try_from(<i32 as NbtDecode>::decode(TagType::Int, reader, tracker)?)
                        .unwrap_or(0);
                tracker.alloc(len * std::mem::size_of::<i64>())?;
                let mut result = Vec::with_capacity(len);
                for _ in 0..len {
                    result.push(<i64 as NbtDecode>::decode(TagType::Long, reader, tracker)?);
                }
                Ok(Tag::LongArray(result))
            }
        }
    }
}

impl NbtDecode for NamedTag {
    type Repr = NamedTag;

    fn decode<R: Read + Seek, Re: ResourceTracker>(
        tag_type: TagType,
        reader: &mut R,
        tracker: &mut Re,
    ) -> Result<Self::Repr, NbtDecodeErr> {
        let name = <String as NbtDecode>::decode(TagType::String, reader, tracker)?;
        let payload = Tag::decode(tag_type, reader, tracker)?;
        Ok(NamedTag { name, payload })
    }
}
