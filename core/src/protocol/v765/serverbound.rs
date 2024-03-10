use std::{
    any::Any,
    io::{Read, Seek, Write},
};

use serverx_macros::{Packet, ProtoDecode, ProtoEncode};
use uuid::Uuid;

use crate::{
    protocol,
    protocol::{
        types::{RemainingBytes, VarInt},
        v765::types::{ChatMode, HandshakeNextState, MainHand, ResourcePackResult},
    },
    utils::identifier::Identifier,
};

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ServerBound, Handshake)]
pub struct HandshakeRequest {
    #[proto(repr = "VarInt")]
    pub version: i32,
    #[proto(max_len = 255)]
    pub server_addr: String,
    pub server_port: u16,
    pub next_state: HandshakeNextState,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ServerBound, Status)]
pub struct StatusRequest;

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ServerBound, Status)]
pub struct StatusPingRequest {
    pub payload: i64,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ServerBound, Login)]
pub struct LoginStart {
    #[proto(max_len = 16)]
    pub name: String,
    pub uuid: Uuid,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ServerBound, Login)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x02, ServerBound, Login)]
pub struct LoginPluginResponse {
    #[proto(repr = "VarInt")]
    pub message_id: i32,
    pub successful: bool,
    #[proto(repr = "RemainingBytes")]
    pub data: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x03, ServerBound, Login)]
pub struct LoginAck;

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ServerBound, Configuration)]
pub struct ConfigClientInformation {
    #[proto(max_len = 16)]
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: MainHand,
    pub text_filtering: u8,
    pub server_listings: u8,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ServerBound, Configuration)]
pub struct ConfigServerBoundPluginMessage {
    pub channel: Identifier,
    #[proto(repr = "RemainingBytes")]
    pub data: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x02, ServerBound, Configuration)]
pub struct ClientFinishConfiguration;

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x03, ServerBound, Configuration)]
pub struct ConfigKeepAlive {
    id: i64,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x04, ServerBound, Configuration)]
pub struct ConfigPong {
    id: i32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x05, ServerBound, Configuration)]
pub struct ConfigResourcePackResponse {
    uuid: Uuid,
    result: ResourcePackResult,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x0b, ServerBound, Play)]
pub struct ConfigurationAck;
