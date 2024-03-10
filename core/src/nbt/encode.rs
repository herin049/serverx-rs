use std::{
    fmt::{Debug, Display, Formatter},
    io,
    io::{Seek, Write},
};

use crate::nbt::{decode::NbtDecodeErr, tag::TagType, NamedTag, Tag};

pub enum NbtEncodeErr {
    IoErr(io::Error),
    InvalidSeqLen(usize),
    UnexpectedTagType(TagType),
}

impl Display for NbtEncodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtEncodeErr::IoErr(err) => write!(f, "io error: {}", err),
            NbtEncodeErr::InvalidSeqLen(len) => write!(f, "invalid sequence length: {}", len),
            NbtEncodeErr::UnexpectedTagType(tag) => write!(f, "unexpected tag type: {}", tag),
        }
    }
}

impl Debug for NbtEncodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

pub trait NbtEncode {
    type Repr;

    fn encode_type<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<TagType, NbtEncodeErr>;
    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), NbtEncodeErr>;
}

macro_rules! number_impl {
    ($($num_ty:ty:$nbt_ty:expr),*) => {
        $(
            impl NbtEncode for $num_ty {
                type Repr = Self;

                fn encode_type<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<TagType, NbtEncodeErr> {
                    <i8 as NbtEncode>::encode(&$nbt_ty.into(), writer)?;
                    Ok($nbt_ty)
                }

                fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), NbtEncodeErr> {
                    writer.write_all(data.to_be_bytes().as_slice()).map_err(|err| NbtEncodeErr::IoErr(err))
                }
            }
        )*
    }
}

number_impl! { i8: TagType::Byte, i16: TagType::Short, i32: TagType::Int, i64: TagType::Long, f32: TagType::Float, f64: TagType::Double }

impl NbtEncode for String {
    type Repr = String;

    fn encode_type<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<TagType, NbtEncodeErr> {
        <i8 as NbtEncode>::encode(&TagType::String.into(), writer)?;
        Ok(TagType::String)
    }

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), NbtEncodeErr> {
        let len = u16::try_from(data.len()).map_err(|_| NbtEncodeErr::InvalidSeqLen(data.len()))?;
        writer
            .write_all(len.to_be_bytes().as_slice())
            .map_err(|err| NbtEncodeErr::IoErr(err))?;
        writer
            .write_all(data.as_bytes())
            .map_err(|err| NbtEncodeErr::IoErr(err))
    }
}

impl NbtEncode for Tag {
    type Repr = Tag;

    fn encode_type<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<TagType, NbtEncodeErr> {
        let tag_type = TagType::of(data);
        <i8 as NbtEncode>::encode(&tag_type.into(), writer)?;
        Ok(tag_type)
    }

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), NbtEncodeErr> {
        match data {
            Tag::End => Ok(()),
            Tag::Byte(value) => <i8 as NbtEncode>::encode(value, writer),
            Tag::Short(value) => <i16 as NbtEncode>::encode(value, writer),
            Tag::Int(value) => <i32 as NbtEncode>::encode(value, writer),
            Tag::Long(value) => <i64 as NbtEncode>::encode(value, writer),
            Tag::Float(value) => <f32 as NbtEncode>::encode(value, writer),
            Tag::Double(value) => <f64 as NbtEncode>::encode(value, writer),
            Tag::ByteArray(value) => {
                let len = i32::try_from(value.len())
                    .map_err(|_| NbtEncodeErr::InvalidSeqLen(value.len()))?;
                <i32 as NbtEncode>::encode(&len, writer)?;
                writer
                    .write_all(value.as_slice())
                    .map_err(|err| NbtEncodeErr::IoErr(err))
            }
            Tag::String(value) => <String as NbtEncode>::encode(value, writer),
            Tag::List(value) => {
                let len = i32::try_from(value.len())
                    .map_err(|_| NbtEncodeErr::InvalidSeqLen(value.len()))?;
                let list_type = value
                    .first()
                    .map(|t| TagType::of(t))
                    .unwrap_or(TagType::End);
                <i8 as NbtEncode>::encode(&list_type.into(), writer)?;
                <i32 as NbtEncode>::encode(&len, writer)?;
                for v in value {
                    Tag::encode(v, writer)?;
                }
                Ok(())
            }
            Tag::Compound(value) => {
                for v in value {
                    <i8 as NbtEncode>::encode(&TagType::of(&v.payload).into(), writer)?;
                    NamedTag::encode(v, writer)?;
                }
                <i8 as NbtEncode>::encode(&TagType::End.into(), writer)
            }
            Tag::IntArray(value) => {
                let len = i32::try_from(value.len())
                    .map_err(|_| NbtEncodeErr::InvalidSeqLen(value.len()))?;
                <i32 as NbtEncode>::encode(&len, writer)?;
                for v in value {
                    <i32 as NbtEncode>::encode(v, writer)?;
                }
                Ok(())
            }
            Tag::LongArray(value) => {
                let len = i32::try_from(value.len())
                    .map_err(|_| NbtEncodeErr::InvalidSeqLen(value.len()))?;
                <i32 as NbtEncode>::encode(&len, writer)?;
                for v in value {
                    <i64 as NbtEncode>::encode(v, writer)?;
                }
                Ok(())
            }
        }
    }
}

impl NbtEncode for NamedTag {
    type Repr = NamedTag;

    fn encode_type<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<TagType, NbtEncodeErr> {
        Tag::encode_type(&data.payload, writer)
    }

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), NbtEncodeErr> {
        <String as NbtEncode>::encode(&data.name, writer)?;
        Tag::encode(&data.payload, writer)
    }
}
