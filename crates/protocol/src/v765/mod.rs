use std::io::{Read, Seek, Write};

use crate as protocol;
use crate::{
    decode::{AllocTracker, ProtoDecodeErr},
    encode::ProtoEncodeErr,
    packet::{
        decode_packet_impl, encode_packet_impl, ConnectionState, Packet, PacketDecoder,
        PacketDirection, PacketEncoder,
    },
    v765::{clientbound::*, serverbound::*},
};
pub const PROTO_NAME: &'static str = "1.20.4";
pub const PROTO_VER: i32 = 765;

pub mod clientbound;
pub mod serverbound;
pub mod types;

pub struct PacketEncoderImpl;

impl PacketEncoder for PacketEncoderImpl {
    fn encode_packet<W: Write + Seek>(
        packet: &dyn Packet,
        id: i32,
        direction: PacketDirection,
        state: ConnectionState,
        writer: &mut W,
    ) -> Result<(), ProtoEncodeErr> {
        match direction {
            PacketDirection::ClientBound => match state {
                ConnectionState::Handshake => {
                    encode_packet_impl!(packet, id, writer,)
                }
                ConnectionState::Status => {
                    encode_packet_impl!(packet, id, writer, StatusResponse, StatusPingResponse)
                }
                ConnectionState::Login => {
                    encode_packet_impl!(
                        packet,
                        id,
                        writer,
                        LoginDisconnect,
                        EncryptionRequest,
                        LoginSuccess,
                        SetCompression,
                        LoginPluginRequest
                    )
                }
                ConnectionState::Play => {
                    encode_packet_impl!(packet, id, writer,)
                }
                ConnectionState::Configuration => {
                    encode_packet_impl!(
                        packet,
                        id,
                        writer,
                        ConfigClientBoundPluginMessage,
                        ConfigDisconnect,
                        ServerFinishConfiguration,
                        clientbound::ConfigKeepAlive,
                        ConfigPing,
                        RegistryData,
                        ConfigRemoveResourcePack,
                        ConfigAddResourcePack,
                        FeatureFlags,
                        UpdateTags
                    )
                }
            },
            PacketDirection::ServerBound => match state {
                ConnectionState::Handshake => {
                    encode_packet_impl!(packet, id, writer, HandshakeRequest)
                }
                ConnectionState::Status => {
                    encode_packet_impl!(packet, id, writer, StatusRequest, StatusPingRequest)
                }
                ConnectionState::Login => {
                    encode_packet_impl!(
                        packet,
                        id,
                        writer,
                        LoginStart,
                        EncryptionResponse,
                        LoginPluginResponse,
                        LoginAck
                    )
                }
                ConnectionState::Play => {
                    encode_packet_impl!(packet, id, writer,)
                }
                ConnectionState::Configuration => {
                    encode_packet_impl!(
                        packet,
                        id,
                        writer,
                        ConfigClientInformation,
                        ConfigServerBoundPluginMessage,
                        ClientFinishConfiguration,
                        serverbound::ConfigKeepAlive,
                        ConfigPong,
                        ConfigResourcePackResponse
                    )
                }
            },
        }
    }
}

pub struct PacketDecoderImpl;

impl PacketDecoder for PacketDecoderImpl {
    fn decode_packet<R: Read + Seek, T: AllocTracker>(
        id: i32,
        direction: PacketDirection,
        state: ConnectionState,
        reader: &mut R,
        alloc_tracker: &mut T,
    ) -> Result<Box<dyn Packet>, ProtoDecodeErr> {
        match direction {
            PacketDirection::ClientBound => match state {
                ConnectionState::Handshake => {
                    decode_packet_impl!(id, reader, alloc_tracker,)
                }
                ConnectionState::Status => {
                    decode_packet_impl!(
                        id,
                        reader,
                        alloc_tracker,
                        StatusResponse,
                        StatusPingResponse
                    )
                }
                ConnectionState::Login => {
                    decode_packet_impl!(
                        id,
                        reader,
                        alloc_tracker,
                        LoginDisconnect,
                        EncryptionRequest,
                        LoginSuccess,
                        SetCompression,
                        LoginPluginRequest
                    )
                }
                ConnectionState::Play => {
                    decode_packet_impl!(id, reader, alloc_tracker,)
                }
                ConnectionState::Configuration => {
                    decode_packet_impl!(
                        id,
                        reader,
                        alloc_tracker,
                        ConfigClientBoundPluginMessage,
                        ConfigDisconnect,
                        ServerFinishConfiguration,
                        serverbound::ConfigKeepAlive,
                        ConfigPing,
                        RegistryData,
                        ConfigRemoveResourcePack,
                        ConfigAddResourcePack,
                        FeatureFlags,
                        UpdateTags
                    )
                }
            },
            PacketDirection::ServerBound => match state {
                ConnectionState::Handshake => {
                    decode_packet_impl!(id, reader, alloc_tracker, HandshakeRequest)
                }
                ConnectionState::Status => {
                    decode_packet_impl!(id, reader, alloc_tracker, StatusRequest, StatusPingRequest)
                }
                ConnectionState::Login => {
                    decode_packet_impl!(
                        id,
                        reader,
                        alloc_tracker,
                        LoginStart,
                        EncryptionResponse,
                        LoginPluginResponse,
                        LoginAck
                    )
                }
                ConnectionState::Play => {
                    decode_packet_impl!(id, reader, alloc_tracker,)
                }
                ConnectionState::Configuration => {
                    decode_packet_impl!(
                        id,
                        reader,
                        alloc_tracker,
                        ConfigClientInformation,
                        ConfigServerBoundPluginMessage,
                        ClientFinishConfiguration,
                        serverbound::ConfigKeepAlive,
                        ConfigPong,
                        ConfigResourcePackResponse
                    )
                }
            },
        }
    }
}
