use std::{
    cmp,
    fmt::{Debug, Display, Formatter},
    io,
    io::{Cursor, IoSliceMut, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
};

use aes::{cipher::BlockDecryptMut, Aes128};
use cfb8::{cipher::AsyncStreamCipher, Decryptor, Encryptor};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    decode::{BasicAllocTracker, ProtoDecode, ProtoDecodeErr},
    encode::{ProtoEncode, ProtoEncodeErr},
    packet::{ConnectionState, Packet, PacketDecoder, PacketDirection, PacketEncoder},
    types::{VarInt, MAX_VAR_INT_LEN},
};

pub const DEFAULT_PACKET_LIMIT: usize = 1 << 21 - 1;
pub const DEFAULT_ALLOC_LIMIT: usize = 1 << 23;

struct VecWriter<'a> {
    vec: &'a mut Vec<u8>,
    offset: usize,
    limit: usize,
}

impl<'a> VecWriter<'a> {
    pub fn new(vec: &'a mut Vec<u8>, offset: usize, limit: usize) -> Self {
        Self { vec, offset, limit }
    }
}

impl<'a> Write for VecWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.offset + buf.len() > self.limit {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        } else if self.offset + buf.len() > self.vec.len() {
            self.vec.resize(self.offset + buf.len(), 0u8);
        }
        &self.vec[self.offset..(self.offset + buf.len())].copy_from_slice(buf);
        self.offset += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Seek for VecWriter<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_offset: usize = match pos {
            SeekFrom::Start(offset) => {
                usize::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?
            }
            SeekFrom::End(offset) => usize::try_from(self.limit as i64 + offset)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?,
            SeekFrom::Current(offset) => usize::try_from(self.offset as i64 + offset)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?,
        };
        if new_offset > self.vec.len() {
            self.vec.resize(cmp::min(new_offset, self.limit), 0u8);
        }
        self.offset = new_offset;
        Ok(self.offset as u64)
    }
}

struct VecReader<'a> {
    vec: &'a Vec<u8>,
    offset: usize,
    end: usize,
}

impl<'a> VecReader<'a> {
    pub fn new(vec: &'a Vec<u8>, offset: usize, end: usize) -> Self {
        Self { vec, offset, end }
    }
}

impl<'a> Read for VecReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset + buf.len() <= self.end {
            let slice = &self.vec[(self.offset)..(self.offset + buf.len())];
            buf.copy_from_slice(slice);
            self.offset += buf.len();
            Ok(buf.len())
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }
}

impl<'a> Seek for VecReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_offset: usize = match pos {
            SeekFrom::Start(offset) => {
                usize::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?
            }
            SeekFrom::End(offset) => usize::try_from(self.end as i64 + offset)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?,
            SeekFrom::Current(offset) => usize::try_from(self.offset as i64 + offset)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?,
        };
        self.offset = new_offset;
        Ok(self.offset as u64)
    }
}

pub enum PacketWriteErr {
    IoErr(io::Error),
    EncodeErr(ProtoEncodeErr),
    EncryptionErr,
    PacketTooLong(usize, usize),
    Unknown,
}

impl Debug for PacketWriteErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketWriteErr::IoErr(err) => write!(f, "io error: {}", err),
            PacketWriteErr::EncodeErr(err) => write!(f, "encode error: {}", err),
            PacketWriteErr::EncryptionErr => write!(f, "encryption err"),
            PacketWriteErr::PacketTooLong(len, max_len) => write!(
                f,
                "packet with length {} is more than the maximum length {}",
                len, max_len
            ),
            PacketWriteErr::Unknown => write!(f, "unknown"),
        }
    }
}

impl Display for PacketWriteErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

pub enum PacketReadErr {
    IoErr(io::Error),
    DecodeErr(ProtoDecodeErr),
    DecryptionErr,
    PacketTooLong(usize, usize),
    MalformedPacketHeader,
}

impl Debug for PacketReadErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketReadErr::IoErr(err) => write!(f, "io error: {}", err),
            PacketReadErr::DecodeErr(err) => write!(f, "decode error: {}", err),
            PacketReadErr::DecryptionErr => write!(f, "encryption err"),
            PacketReadErr::PacketTooLong(len, max_len) => write!(
                f,
                "packet with length {} is more than the maximum length {}",
                len, max_len
            ),
            PacketReadErr::MalformedPacketHeader => write!(f, "malformed packet header"),
        }
    }
}

impl Display for PacketReadErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

pub struct AsyncPacketWriter {
    packet_buf: Vec<u8>,
    deflate_buf: Vec<u8>,
    encryptor: Option<Encryptor<Aes128>>,
    crypt_key: [u8; 16],
    compression: Option<usize>,
    packet_limit: usize,
}

impl AsyncPacketWriter {
    pub fn new() -> Self {
        Self {
            packet_buf: Vec::<u8>::new(),
            deflate_buf: Vec::<u8>::new(),
            encryptor: None,
            crypt_key: [0u8; 16],
            compression: None,
            packet_limit: DEFAULT_PACKET_LIMIT,
        }
    }

    pub async fn write_frame<W: AsyncWrite + Unpin>(
        &mut self,
        writer: &mut W,
        len: usize,
    ) -> Result<(), PacketWriteErr> {
        if len > self.packet_limit {
            Err(PacketWriteErr::PacketTooLong(len, self.packet_limit))
        } else if len > self.packet_buf.len() {
            Err(PacketWriteErr::Unknown)
        } else {
            let mut encoded_len = [0u8; MAX_VAR_INT_LEN];
            let p = {
                let mut cursor = Cursor::new(encoded_len.as_mut_slice());
                VarInt::encode(&(len as i32), &mut cursor)
                    .map_err(|err| PacketWriteErr::EncodeErr(err))?;
                cursor.position() as usize
            };
            let buf_offset = MAX_VAR_INT_LEN - p;
            (&mut self.packet_buf[buf_offset..MAX_VAR_INT_LEN]).copy_from_slice(&encoded_len[..p]);
            let data_slice = self
                .packet_buf
                .as_mut_slice()
                .get_mut(buf_offset..(MAX_VAR_INT_LEN + len))
                .ok_or(PacketWriteErr::Unknown)?;
            if let Some(encryptor) = self.encryptor.as_mut() {
                panic!("encryption not supported");
            }
            writer
                .write_all(data_slice)
                .await
                .map_err(|err| PacketWriteErr::IoErr(err))
        }
    }

    pub async fn write<W: AsyncWrite + Unpin, S: PacketEncoder>(
        &mut self,
        writer: &mut W,
        direction: PacketDirection,
        state: ConnectionState,
        packet: &dyn Packet,
    ) -> Result<(), PacketWriteErr> {
        let mut vec_writer =
            VecWriter::new(&mut self.packet_buf, MAX_VAR_INT_LEN, self.packet_limit);
        let packet_id = packet.id();
        VarInt::encode(&packet_id, &mut vec_writer)
            .map_err(|err| PacketWriteErr::EncodeErr(err))?;
        S::encode_packet(packet, packet_id, direction, state, &mut vec_writer)
            .map_err(|err| PacketWriteErr::EncodeErr(err))?;
        let packet_len = vec_writer.offset - MAX_VAR_INT_LEN;
        self.write_frame(writer, packet_len).await
    }
}

pub struct AsyncPacketReader {
    packet_buf: Vec<u8>,
    inflate_buf: Vec<u8>,
    decryptor: Option<Decryptor<Aes128>>,
    crypt_key: [u8; 16],
    compression: Option<usize>,
    packet_limit: usize,
}

impl AsyncPacketReader {
    pub fn new() -> Self {
        Self {
            packet_buf: Vec::<u8>::new(),
            inflate_buf: Vec::<u8>::new(),
            decryptor: None,
            crypt_key: [0u8; 16],
            compression: None,
            packet_limit: DEFAULT_PACKET_LIMIT,
        }
    }

    pub async fn read_frame_size<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut R,
    ) -> Result<usize, PacketReadErr> {
        let mut size: i32 = 0;
        for i in 0..5 {
            let mut bytes = [0u8];
            reader
                .read_exact(bytes.as_mut_slice())
                .await
                .map_err(|err| PacketReadErr::IoErr(err))?;
            if let Some(decryptor) = self.decryptor.as_mut() {
                panic!("encryption not supported");
            }
            let b: i32 = i32::from(bytes[0]);
            size |= (b & 0x7f) << (i * 7);
            if (b >> 7) == 0 {
                return Ok(usize::try_from(size).map_err(|_| PacketReadErr::MalformedPacketHeader)?);
            }
        }
        Err(PacketReadErr::MalformedPacketHeader)
    }

    pub async fn read_frame<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut R,
    ) -> Result<usize, PacketReadErr> {
        if self.compression.is_some() {
            panic!("compression is not supported");
        } else {
            let frame_size = self.read_frame_size(reader).await?;
            if frame_size > self.packet_limit {
                return Err(PacketReadErr::PacketTooLong(frame_size, self.packet_limit));
            } else if self.packet_buf.len() < frame_size {
                self.packet_buf.resize(frame_size, 0u8);
            }
            let slice = self
                .packet_buf
                .as_mut_slice()
                .get_mut(0..frame_size)
                .ok_or_else(|| PacketReadErr::MalformedPacketHeader)?;
            reader
                .read_exact(slice)
                .await
                .map_err(|err| PacketReadErr::IoErr(err))?;
            Ok(frame_size)
        }
    }

    pub async fn read<R: AsyncRead + Unpin, D: PacketDecoder>(
        &mut self,
        reader: &mut R,
        direction: PacketDirection,
        state: ConnectionState,
    ) -> Result<Box<dyn Packet>, PacketReadErr> {
        let frame_size = self.read_frame(reader).await?;
        let mut vec_reader = VecReader::new(&mut self.packet_buf, 0, frame_size);
        let mut alloc_tracker = BasicAllocTracker::new(DEFAULT_ALLOC_LIMIT);
        let packet_id = VarInt::decode(&mut vec_reader, &mut alloc_tracker)
            .map_err(|err| PacketReadErr::DecodeErr(err))?;
        D::decode_packet(
            packet_id,
            direction,
            state,
            &mut vec_reader,
            &mut alloc_tracker,
        )
        .map_err(|err| PacketReadErr::DecodeErr(err))
    }
}
