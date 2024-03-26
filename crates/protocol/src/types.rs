use std::{
    io::{Read, Seek, SeekFrom, Write},
    iter, slice,
    sync::Arc,
};

use either::Either;
use serverx_common::{collections::bit_vec::BitVec, identifier::Identifier};
use serverx_nbt::{
    decode::{BasicResourceTracker, NbtDecode, NbtDecodeErr},
    encode::{NbtEncode, NbtEncodeErr},
    tag::{TagType, MAX_RECURSION_DEPTH, MAX_TAG_SIZE},
    Tag, TagRoot,
};
use uuid::Uuid;

use crate::{
    decode::{
        AllocTracker, ProtoDecode, ProtoDecodeErr, ProtoDecodeErr::InvalidEnumTag, ProtoDecodeSeq,
    },
    encode::{ProtoEncode, ProtoEncodeErr, ProtoEncodeSeq},
};

macro_rules! number_impl {
    ($($num_ty:ty),*) => {
        $(
            impl ProtoEncode for $num_ty {
                type Repr = Self;

                fn encode<W: Write + Seek>(
                    data: &Self::Repr,
                    writer: &mut W,
                ) -> Result<(), ProtoEncodeErr> {
                    writer
                        .write(data.to_be_bytes().as_slice())
                        .map_err(|err| ProtoEncodeErr::IoErr(err))?;
                    Ok(())
                }
            }

            impl ProtoDecode for $num_ty {
                type Repr = Self;

                fn decode<R: Read + Seek, A: AllocTracker>(
                    reader: &mut R,
                    _alloc_tracker: &mut A,
                ) -> Result<Self::Repr, ProtoDecodeErr> {
                    let mut buf = [0; std::mem::size_of::<Self>()];
                    reader
                        .read_exact(buf.as_mut_slice())
                        .map_err(|err| ProtoDecodeErr::IoErr(err))?;
                    Ok(Self::from_be_bytes(buf))
                }
            }
        )*
    };
}

number_impl! { u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64 }

impl ProtoEncode for bool {
    type Repr = bool;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        u8::encode(&(*data as u8), writer)
    }
}

impl ProtoDecode for bool {
    type Repr = bool;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        u8::decode(reader, alloc_tracker).map(|x| x != 0)
    }
}

pub struct VarInt;

impl VarInt {
    pub const MAX_BYTES: usize = 5;
}

impl ProtoEncode for VarInt {
    type Repr = i32;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let mut x: u32 = *data as u32;
        while (x & !0x7f) != 0 {
            <u8 as ProtoEncode>::encode(&(((x & 0x7f) | 0x80) as u8), writer)?;
            x >>= 7;
        }
        <u8 as ProtoEncode>::encode(&(x as u8), writer)
    }
}

impl ProtoDecode for VarInt {
    type Repr = i32;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        _alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let mut result: Self::Repr = 0;
        for i in 0..VarInt::MAX_BYTES {
            let curr_byte = Self::Repr::from(<u8 as ProtoDecode>::decode(reader, _alloc_tracker)?);
            result |= (curr_byte & 0x7f) << (i * 7);
            if (curr_byte >> 7) == 0 {
                return Ok(result);
            }
        }
        Err(ProtoDecodeErr::VarIntTooLong)
    }
}

pub struct VarLong;

impl VarLong {
    pub const MAX_BYTES: usize = 10;
}

impl ProtoEncode for VarLong {
    type Repr = i64;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let mut x: u64 = *data as u64;
        while (x & !0x7f) != 0 {
            <u8 as ProtoEncode>::encode(&(((x & 0x7f) | 0x80) as u8), writer)?;
            x >>= 7;
        }
        <u8 as ProtoEncode>::encode(&(x as u8), writer)
    }
}

impl ProtoDecode for VarLong {
    type Repr = i64;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        _alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let mut result: Self::Repr = 0;
        for i in 0..VarLong::MAX_BYTES {
            let curr_byte = Self::Repr::from(<u8 as ProtoDecode>::decode(reader, _alloc_tracker)?);
            result |= (curr_byte & 0x7f) << (i * 7);
            if (curr_byte >> 7) == 0 {
                return Ok(result);
            }
        }
        Err(ProtoDecodeErr::VarIntTooLong)
    }
}

impl ProtoEncode for Uuid {
    type Repr = Uuid;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        u128::encode(&data.as_u128(), writer)
    }
}

impl ProtoDecode for Uuid {
    type Repr = Uuid;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        Ok(Uuid::from_u128(u128::decode(reader, alloc_tracker)?))
    }
}

pub struct Angle;

impl ProtoEncode for Angle {
    type Repr = f32;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let clipped = ((data % 360.0) + 360.0) % 360.0;
        let val = (clipped * 256.0 / 360.0) as u8;
        u8::encode(&val, writer)
    }
}

impl ProtoDecode for Angle {
    type Repr = f32;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let val = u8::decode(reader, alloc_tracker)?;
        Ok((val as f32) * 360.0 / 256.0)
    }
}

pub struct Position;

impl ProtoEncode for Position {
    type Repr = (i32, i32, i32);

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let x: u64 = ((data.0 as u64 & 0x3ffffff) << 38)
            | (data.1 as u64 & 0xfff)
            | ((data.2 as u64 & 0x3ffffff) << 12);
        u64::encode(&x, writer)
    }
}

impl ProtoDecode for Position {
    type Repr = (i32, i32, i32);

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let val: i64 = <i64 as ProtoDecode>::decode(reader, alloc_tracker)?;
        let x = (val >> 38) as i32;
        let y = (val & 0xfff) as i32;
        let z = (val << 26 >> 38) as i32;
        Ok((x, y, z))
    }
}

impl<T> ProtoEncode for Option<T>
where
    T: ProtoEncode,
{
    type Repr = Option<T::Repr>;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        if let Some(val) = data {
            T::encode(val, writer)?;
        }
        Ok(())
    }
}

impl<T> ProtoDecode for Option<T>
where
    T: ProtoDecode,
{
    type Repr = Option<T::Repr>;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let present = bool::decode(reader, alloc_tracker)?;
        if present {
            Ok(Some(T::decode(reader, alloc_tracker)?))
        } else {
            Ok(None)
        }
    }
}

impl<L, R> ProtoEncode for Either<L, R>
where
    L: ProtoEncode,
    R: ProtoEncode,
{
    type Repr = Either<L::Repr, R::Repr>;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        match data {
            Either::Left(left) => L::encode(left, writer),
            Either::Right(right) => R::encode(right, writer),
        }
    }
}

impl<X, Y> ProtoDecode for Either<X, Y>
where
    X: ProtoDecode,
    Y: ProtoDecode,
{
    type Repr = Either<X::Repr, Y::Repr>;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        if let Ok(left) = X::decode(reader, alloc_tracker) {
            Ok(Either::Left(left))
        } else if let Ok(right) = Y::decode(reader, alloc_tracker) {
            Ok(Either::Right(right))
        } else {
            Err(ProtoDecodeErr::InvalidEither)
        }
    }
}

impl<T: ProtoEncodeSeq> ProtoEncode for T {
    type Repr = T::Repr;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let len = <T as ProtoEncodeSeq>::encode_len(data, writer)?;
        <T as ProtoEncodeSeq>::encode_seq(data, writer, len)
    }
}

impl<T: ProtoDecodeSeq> ProtoDecode for T {
    type Repr = T::Repr;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let len = <T as ProtoDecodeSeq>::decode_len(reader, alloc_tracker)?;
        <T as ProtoDecodeSeq>::decode_seq(reader, alloc_tracker, len)
    }
}

pub const MAX_VEC_LEN: usize = 1 << 21;

impl<T: ProtoEncode> ProtoEncodeSeq for Vec<T> {
    type Repr = Vec<T::Repr>;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        let data_len = data.len();
        if data_len > MAX_VEC_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(data_len, MAX_VEC_LEN));
        }
        let len = <VarInt as ProtoDecode>::Repr::try_from(data_len)
            .map_err(|_| ProtoEncodeErr::InvalidSeqLen(data_len))?;
        VarInt::encode(&len, writer)?;
        Ok(data_len)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        _len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        for e in data {
            T::encode(e, writer)?;
        }
        Ok(())
    }
}

impl<'a, T: ProtoEncode> ProtoEncodeSeq for &'a [T] {
    type Repr = &'a [T::Repr];

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        let data_len = data.len();
        if data_len > MAX_VEC_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(data_len, MAX_VEC_LEN));
        }
        let len = <VarInt as ProtoEncode>::Repr::try_from(data_len)
            .map_err(|_| ProtoEncodeErr::InvalidSeqLen(data_len))?;
        VarInt::encode(&len, writer)?;
        Ok(data_len)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        for e in *data {
            T::encode(e, writer)?;
        }
        Ok(())
    }
}

impl<'a, T: ProtoEncode> ProtoEncodeSeq for slice::Iter<'a, T> {
    type Repr = slice::Iter<'a, T::Repr>;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        let data_len = data.len();
        if data_len > MAX_VEC_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(data_len, MAX_VEC_LEN));
        }
        let len = <VarInt as ProtoEncode>::Repr::try_from(data_len)
            .map_err(|_| ProtoEncodeErr::InvalidSeqLen(data_len))?;
        VarInt::encode(&len, writer)?;
        Ok(data_len)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        for e in *data {
            T::encode(e, writer)?;
        }
        Ok(())
    }
}

impl<T: ProtoDecode> ProtoDecodeSeq for Vec<T> {
    type Repr = Vec<T::Repr>;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr> {
        let len = <VarInt as ProtoDecode>::decode(reader, alloc_tracker)?;
        let data_len = usize::try_from(len).map_err(|_| ProtoDecodeErr::InvalidSeqLen(len))?;
        if data_len > MAX_VEC_LEN {
            return Err(ProtoDecodeErr::SeqTooLong(data_len, MAX_VEC_LEN));
        }
        Ok(data_len)
    }

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
        size: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        alloc_tracker.alloc(size * std::mem::size_of::<T::Repr>())?;
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(T::decode(reader, alloc_tracker)?);
        }
        Ok(result)
    }
}

pub const MAX_STR_LEN: usize = 32767 * 3;

impl ProtoEncodeSeq for String {
    type Repr = Self;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        let data_len = data.len();
        if data_len > MAX_STR_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(data_len, MAX_STR_LEN));
        }
        let len = <VarInt as ProtoDecode>::Repr::try_from(data_len)
            .map_err(|_| ProtoEncodeErr::InvalidSeqLen(data_len))?;
        VarInt::encode(&len, writer)?;
        Ok(data_len)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        writer
            .write_all(data.as_bytes())
            .map_err(|err| ProtoEncodeErr::IoErr(err))
    }
}

impl ProtoDecodeSeq for String {
    type Repr = Self;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr> {
        let len = <VarInt as ProtoDecode>::decode(reader, alloc_tracker)?;
        let data_len = usize::try_from(len).map_err(|_| ProtoDecodeErr::InvalidSeqLen(len))?;
        if data_len > MAX_STR_LEN {
            return Err(ProtoDecodeErr::SeqTooLong(data_len, MAX_STR_LEN));
        }
        Ok(data_len)
    }

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
        len: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        alloc_tracker.alloc(len * std::mem::size_of::<u8>())?;
        let mut buf = vec![0u8; len];
        reader
            .read_exact(buf.as_mut_slice())
            .map_err(|err| ProtoDecodeErr::IoErr(err))?;
        let str = String::from_utf8(buf).map_err(|_| ProtoDecodeErr::MalformedString)?;
        Ok(str)
    }
}

pub const MAX_JSON_LEN: usize = 262144 * 3;

impl ProtoEncode for serde_json::Value {
    type Repr = serde_json::Value;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let encoded = data.to_string();
        let data_len = encoded.len();
        if data_len > MAX_JSON_LEN {
            return Err(ProtoEncodeErr::SeqTooLong(data_len, MAX_JSON_LEN));
        }
        let len = <VarInt as ProtoDecode>::Repr::try_from(data_len)
            .map_err(|_| ProtoEncodeErr::InvalidSeqLen(data_len))?;
        VarInt::encode(&len, writer)?;
        String::encode_seq(&encoded, writer, data_len)
    }
}

impl ProtoDecode for serde_json::Value {
    type Repr = serde_json::Value;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let len = <VarInt as ProtoDecode>::decode(reader, alloc_tracker)?;
        let data_len = usize::try_from(len).map_err(|_| ProtoDecodeErr::InvalidSeqLen(len))?;
        if data_len > MAX_JSON_LEN {
            return Err(ProtoDecodeErr::SeqTooLong(data_len, MAX_JSON_LEN));
        }
        let json_str = String::decode_seq(reader, alloc_tracker, data_len)?;
        serde_json::from_str(json_str.as_str()).map_err(|_| ProtoDecodeErr::MalformedJson(json_str))
    }
}

pub struct RemainingBytes;

impl ProtoEncodeSeq for RemainingBytes {
    type Repr = Vec<u8>;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        Ok(data.len())
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        writer
            .write_all(data.as_slice())
            .map_err(|err| ProtoEncodeErr::IoErr(err))
    }
}

impl ProtoDecodeSeq for RemainingBytes {
    type Repr = Vec<u8>;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        _alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr> {
        let prev_pos = reader
            .stream_position()
            .map_err(|err| ProtoDecodeErr::IoErr(err))?;
        let end = reader
            .seek(SeekFrom::End(0))
            .map_err(|err| ProtoDecodeErr::IoErr(err))?;
        let len = end - prev_pos;
        if prev_pos != end {
            reader
                .seek(SeekFrom::Start(prev_pos))
                .map_err(|err| ProtoDecodeErr::IoErr(err))?;
        }
        Ok(len as usize)
    }

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        _alloc_tracker: &mut A,
        len: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        Vec::<u8>::decode_seq(reader, _alloc_tracker, len)
    }
}

impl<T: ProtoEncode> ProtoEncode for Arc<T> {
    type Repr = Arc<T::Repr>;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        T::encode(data.as_ref(), writer)
    }
}

impl<T: ProtoDecode> ProtoDecode for Arc<T> {
    type Repr = Arc<T::Repr>;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        Ok(Arc::new(T::decode(reader, alloc_tracker)?))
    }
}
impl ProtoEncode for TagRoot {
    type Repr = Tag;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        let tag_type = TagType::of(data);
        if tag_type != TagType::Compound {
            Err(ProtoEncodeErr::NbtEncodeErr(
                NbtEncodeErr::UnexpectedTagType(tag_type),
            ))
        } else {
            <i8 as ProtoEncode>::encode(&tag_type.into(), writer)?;
            <Tag as NbtEncode>::encode(data, writer)
                .map_err(|err| ProtoEncodeErr::NbtEncodeErr(err))
        }
    }
}

impl ProtoDecode for TagRoot {
    type Repr = Tag;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let tag_type_id = <i8 as ProtoDecode>::decode(reader, alloc_tracker)?;
        let tag_type = TagType::try_from(tag_type_id).map_err(|err| {
            ProtoDecodeErr::NbtDecodeErr(NbtDecodeErr::InvalidTagType(tag_type_id))
        })?;
        if tag_type != TagType::Compound {
            return Err(ProtoDecodeErr::NbtDecodeErr(
                NbtDecodeErr::UnexpectedTagType(tag_type),
            ));
        }
        let allocated_start = alloc_tracker.allocated();
        let mut nbt_resource_tracker = BasicResourceTracker {
            allocated: allocated_start,
            max_allocated: MAX_TAG_SIZE,
            depth: 0,
            max_depth: MAX_RECURSION_DEPTH,
        };
        let result = <Tag as NbtDecode>::decode(tag_type, reader, &mut nbt_resource_tracker)
            .map_err(|err| ProtoDecodeErr::NbtDecodeErr(err))?;
        alloc_tracker.alloc(nbt_resource_tracker.allocated - allocated_start)?;
        Ok(result)
    }
}

impl ProtoEncodeSeq for BitVec {
    type Repr = BitVec;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr> {
        <&[u64] as ProtoEncodeSeq>::encode_len(&data.as_slice(), writer)
    }

    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr> {
        for e in data.as_slice() {
            writer
                .write(e.to_le_bytes().as_slice())
                .map_err(|err| ProtoEncodeErr::IoErr(err))?;
        }
        Ok(())
    }
}

impl ProtoDecodeSeq for BitVec {
    type Repr = BitVec;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr> {
        <Vec<u64> as ProtoDecodeSeq>::decode_len(reader, alloc_tracker)
    }

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
        len: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        alloc_tracker.alloc(len * std::mem::size_of::<u64>())?;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            let mut bytes = [0u8; std::mem::size_of::<u64>()];
            reader
                .read_exact(bytes.as_mut_slice())
                .map_err(|err| ProtoDecodeErr::IoErr(err))?;
            result.push(u64::from_le_bytes(bytes));
        }
        Ok(Self::from_raw_parts(result))
    }
}

impl ProtoEncode for Identifier {
    type Repr = Identifier;

    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr> {
        <String as ProtoEncode>::encode(data.string_ref(), writer)
    }
}

impl ProtoDecode for Identifier {
    type Repr = Identifier;

    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr> {
        let str = <String as ProtoDecode>::decode(reader, alloc_tracker)?;
        Ok(Identifier::try_from(str.as_str()).map_err(|_| ProtoDecodeErr::InvalidIdentifier)?)
    }
}
