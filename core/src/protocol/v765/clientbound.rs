use std::{
    any::Any,
    io::{Read, Seek, Write},
};

use serverx_macros::{Packet, ProtoDecode, ProtoEncode};
use uuid::Uuid;

use crate::{
    nbt, protocol,
    protocol::{
        types::{RemainingBytes, VarInt},
        v765::types::LoginProperty,
    },
    utils::identifier::Identifier,
};

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ClientBound, Status)]
pub struct StatusResponse {
    pub response: serde_json::Value,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ClientBound, Status)]
pub struct StatusPingResponse {
    pub payload: i64,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ClientBound, Login)]
pub struct LoginDisconnect {
    pub reason: serde_json::Value,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ClientBound, Login)]
pub struct EncryptionRequest {
    #[proto(max_len = 20)]
    pub server_id: String,
    pub public_key: Vec<u8>,
    #[proto(exact_len = 4)]
    pub verify_token: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x02, ClientBound, Login)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    #[proto(max_len = 16)]
    pub username: String,
    pub properties: Vec<LoginProperty>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x03, ClientBound, Login)]
pub struct SetCompression {
    #[proto(repr = "VarInt")]
    pub threshold: i32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x04, ClientBound, Login)]
pub struct LoginPluginRequest {
    #[proto(repr = "VarInt")]
    pub message_id: i32,
    pub channel: String,
    #[proto(repr = "RemainingBytes")]
    pub data: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x00, ClientBound, Configuration)]
pub struct ConfigClientBoundPluginMessage {
    pub channel: Identifier,
    #[proto(repr = "RemainingBytes")]
    pub data: Vec<u8>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x01, ClientBound, Configuration)]
pub struct ConfigDisconnect {
    pub reason: String,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x02, ClientBound, Configuration)]
pub struct ServerFinishConfiguration;

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x03, ClientBound, Configuration)]
pub struct ConfigKeepAlive {
    pub keep_alive_id: i64,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x04, ClientBound, Configuration)]
pub struct ConfigPing {
    pub id: i32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x05, ClientBound, Configuration)]
pub struct RegistryData {
    #[proto(repr = "nbt::TagRoot")]
    pub registries: nbt::Tag,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x06, ClientBound, Configuration)]
pub struct ConfigRemoveResourcePack {
    pub uuid: Option<Uuid>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x07, ClientBound, Configuration)]
pub struct ConfigAddResourcePack {
    pub uuid: Option<Uuid>,
    #[proto(max_len = 32767)]
    pub url: String,
    #[proto(max_len = 40)]
    pub hash: String,
    pub forced: bool,
    pub message: Option<String>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x08, ClientBound, Configuration)]
pub struct FeatureFlags {
    pub flags: Vec<Identifier>,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x67, ClientBound, Play)]
pub struct StartConfiguration;
