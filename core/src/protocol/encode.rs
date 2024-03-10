use std::{
    fmt::{Debug, Display, Formatter},
    io,
    io::{Seek, Write},
};

use crate::nbt::encode::NbtEncodeErr;

pub enum ProtoEncodeErr {
    IoErr(io::Error),
    InvalidSeqLen(usize),
    SeqTooShort(usize, usize),
    SeqTooLong(usize, usize),
    SeqLenMismatch(usize, usize),
    UnknownPacketId(i32),
    MalformedPacket,
    NbtEncodeErr(NbtEncodeErr),
}

impl Display for ProtoEncodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoEncodeErr::IoErr(err) => write!(f, "io error: {}", err),
            ProtoEncodeErr::InvalidSeqLen(len) => write!(f, "invalid sequence length: {}", len),
            ProtoEncodeErr::SeqTooShort(len, min_len) => write!(
                f,
                "sequence with length {} is less than the minimum length {}",
                len, min_len
            ),
            ProtoEncodeErr::SeqTooLong(len, max_len) => write!(
                f,
                "sequence with length {} is more than the maximum length {}",
                len, max_len
            ),
            ProtoEncodeErr::SeqLenMismatch(len, expected_len) => write!(
                f,
                "sequence with length {} is not equal to the expected length {}",
                len, expected_len
            ),
            ProtoEncodeErr::UnknownPacketId(id) => write!(f, "unknown packet id {}", id),
            ProtoEncodeErr::MalformedPacket => write!(f, "malformed packet"),
            ProtoEncodeErr::NbtEncodeErr(err) => write!(f, "nbt encode error: {}", err),
        }
    }
}

impl Debug for ProtoEncodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

pub trait ProtoEncode {
    type Repr;
    fn encode<W: Write + Seek>(data: &Self::Repr, writer: &mut W) -> Result<(), ProtoEncodeErr>;
}

pub trait ProtoEncodeSeq {
    type Repr;

    fn encode_len<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
    ) -> Result<usize, ProtoEncodeErr>;
    fn encode_seq<W: Write + Seek>(
        data: &Self::Repr,
        writer: &mut W,
        len: usize,
    ) -> Result<(), ProtoEncodeErr>;
}
