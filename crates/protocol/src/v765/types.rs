use std::io::{Read, Seek, Write};
use serverx_common::identifier::Identifier;

use serverx_macros::{ProtoDecode, ProtoEncode};

use crate::types::*;
use crate as protocol;

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(enum_repr = "VarInt")]
pub enum HandshakeNextState {
    #[proto(tag = 1)]
    Status,
    #[proto(tag = 2)]
    Login,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct LoginProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub enum ChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub enum MainHand {
    Left,
    Right,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub enum ResourcePackResult {
    Success,
    Declined,
    Failed,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedToReload,
    Discarded,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct RegistryTag {
    pub registry: Identifier,
    pub entries: Vec<RegistryEntry>,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct RegistryEntry {
    pub tag_name: Identifier,
    #[proto(repr = "Vec<VarInt>")]
    pub tag_entries: Vec<i32>,
}
