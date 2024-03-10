use std::{
    any::Any,
    fmt::Debug,
    io::{Read, Seek, Write},
};

use crate::protocol::{
    decode::{AllocTracker, ProtoDecode, ProtoDecodeErr},
    encode::{ProtoEncode, ProtoEncodeErr},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PacketDirection {
    ClientBound,
    ServerBound,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]

pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
    Configuration,
}

pub trait Packet: Send + Sync + Debug {
    fn id(&self) -> i32;
    fn direction(&self) -> PacketDirection;
    fn state(&self) -> ConnectionState;
    fn as_any(&self) -> &(dyn Any + Send + Sync);
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

pub trait PacketEncoder {
    fn encode_packet<W: Write + Seek>(
        packet: &dyn Packet,
        id: i32,
        direction: PacketDirection,
        state: ConnectionState,
        writer: &mut W,
    ) -> Result<(), ProtoEncodeErr>;
}

pub trait PacketDecoder {
    fn decode_packet<R: Read + Seek, T: AllocTracker>(
        id: i32,
        direction: PacketDirection,
        state: ConnectionState,
        reader: &mut R,
        alloc_tracker: &mut T,
    ) -> Result<Box<dyn Packet>, ProtoDecodeErr>;
}

macro_rules! encode_packet_impl {
    ($packet:ident,$id: ident,$writer:ident,$($packet_ty:ty),*) => {
        match $id {
            $(<$packet_ty>::ID => {
                <$packet_ty as protocol::encode::ProtoEncode>::encode($packet.as_any().downcast_ref::<$packet_ty>().ok_or_else(|| protocol::encode::ProtoEncodeErr::MalformedPacket)?, $writer)
            })*
            _ => Err(protocol::encode::ProtoEncodeErr::UnknownPacketId($id))
        }
    }
}

pub(crate) use encode_packet_impl;

macro_rules! decode_packet_impl {
    ($id: ident, $reader:ident, $alloc_tracker:ident,$($packet_ty:ty),*) => {
        match $id {
            $(<$packet_ty>::ID => {
                Ok(Box::new(<$packet_ty as protocol::decode::ProtoDecode>::decode($reader, $alloc_tracker)?))
            })*
            _ => Err(protocol::decode::ProtoDecodeErr::UnknownPacketId($id))
        }
    }
}

pub(crate) use decode_packet_impl;
