use std::{
    fmt::{Debug, Display, Formatter},
    io,
    io::{Read, Seek},
};

use serverx_nbt::decode::NbtDecodeErr;

pub trait AllocTracker {
    fn alloc(&mut self, count: usize) -> Result<(), ProtoDecodeErr>;
    fn dealloc(&mut self, count: usize);
    fn allocated(&self) -> usize;
}

pub struct BasicAllocTracker {
    allocated: usize,
    limit: usize,
}

impl BasicAllocTracker {
    pub fn new(limit: usize) -> Self {
        Self {
            allocated: 0,
            limit,
        }
    }
}

impl AllocTracker for BasicAllocTracker {
    fn alloc(&mut self, count: usize) -> Result<(), ProtoDecodeErr> {
        if self.allocated + count > self.limit {
            Err(ProtoDecodeErr::MemoryLimitExceeded(self.limit))
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
}

pub enum ProtoDecodeErr {
    IoErr(io::Error),
    InvalidSeqLen(i32),
    MemoryLimitExceeded(usize),
    VarIntTooLong,
    VarLongTooLong,
    SeqTooShort(usize, usize),
    SeqTooLong(usize, usize),
    SeqLenMismatch(usize, usize),
    MalformedString,
    InvalidEnumTag,
    InvalidEither,
    UnknownPacketId(i32),
    MalformedPacket,
    MalformedJson(String),
    InvalidIdentifier,
    NbtDecodeErr(NbtDecodeErr),
}

impl Display for ProtoDecodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoDecodeErr::IoErr(err) => write!(f, "io error: {}", err),
            ProtoDecodeErr::InvalidSeqLen(len) => write!(f, "invalid sequence length: {}", len),
            ProtoDecodeErr::MemoryLimitExceeded(limit) => {
                write!(f, "memory limit of {} bytes exceeded", limit)
            }
            ProtoDecodeErr::VarIntTooLong => write!(f, "VarInt exceeded maximum length of 5 bytes"),
            ProtoDecodeErr::VarLongTooLong => {
                write!(f, "VarLong exceeded maximum length of 10 bytes")
            }
            ProtoDecodeErr::SeqTooShort(len, min_len) => write!(
                f,
                "sequence with length {} is less than the minimum length {}",
                len, min_len
            ),
            ProtoDecodeErr::SeqTooLong(len, max_len) => write!(
                f,
                "sequence with length {} is more than the maximum length {}",
                len, max_len
            ),
            ProtoDecodeErr::SeqLenMismatch(len, expected_len) => write!(
                f,
                "sequence with length {} is not equal to the expected length {}",
                len, expected_len
            ),
            ProtoDecodeErr::MalformedString => write!(f, "malformed string"),
            ProtoDecodeErr::InvalidEnumTag => write!(f, "invalid enum tag encountered"),
            ProtoDecodeErr::InvalidEither => write!(f, "unable to decode either type"),
            ProtoDecodeErr::UnknownPacketId(id) => write!(f, "unknown packet id {}", id),
            ProtoDecodeErr::MalformedPacket => write!(f, "malformed packet"),
            ProtoDecodeErr::MalformedJson(json_str) => write!(f, "malformed json: {}", json_str),
            ProtoDecodeErr::InvalidIdentifier => write!(f, "invalid identifier"),
            ProtoDecodeErr::NbtDecodeErr(err) => write!(f, "nbt decode error: {}", err),
        }
    }
}

impl Debug for ProtoDecodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

pub trait ProtoDecode {
    type Repr;
    fn decode<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<Self::Repr, ProtoDecodeErr>;
}

pub trait ProtoDecodeSeq {
    type Repr;

    fn decode_len<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
    ) -> Result<usize, ProtoDecodeErr>;

    fn decode_seq<R: Read + Seek, A: AllocTracker>(
        reader: &mut R,
        alloc_tracker: &mut A,
        len: usize,
    ) -> Result<Self::Repr, ProtoDecodeErr>;
}
