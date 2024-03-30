use std::io::{Read, Seek, Write};

use serverx_common::{collections::bit_vec::BitVec, identifier::Identifier};
use serverx_macros::{ProtoDecode, ProtoEncode};
use serverx_nbt as nbt;

use crate as protocol;
use crate::types::*;

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(tag_repr = "VarInt")]
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

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct BlockEntityRecord {
    pub pos: u8,
    pub height: i16,
    #[proto(repr = "VarInt")]
    pub entity_type: i32,
    #[proto(repr = "nbt::TagRoot")]
    pub entity_data: nbt::Tag,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct ChunkLighting {
    pub sky_light_mask: BitVec,
    pub block_light_mask: BitVec,
    pub empty_sky_light_mask: BitVec,
    pub empty_block_light_mask: BitVec,
    pub sky_light_sections: Vec<LightingArray>,
    pub block_light_sections: Vec<LightingArray>,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct LightingArray {
    #[proto(exact_len = 2048)]
    pub data: Vec<u8>,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(tag_repr = "u8")]
pub enum GameMode {
    #[proto(tag = 0)]
    Survival,
    #[proto(tag = 1)]
    Creative,
    #[proto(tag = 2)]
    Adventure,
    #[proto(tag = 3)]
    Spectator,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(tag_repr = "i8")]
pub enum LastGameMode {
    #[proto(tag = -1)]
    Undefined,
    #[proto(tag = 0)]
    Survival,
    #[proto(tag = 1)]
    Creative,
    #[proto(tag = 2)]
    Adventure,
    #[proto(tag = 3)]
    Spectator,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
pub struct DeathLocation {
    pub dimension: Identifier,
    #[proto(repr = "Position")]
    pub position: (i32, i32, i32),
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(tag_repr = "u8")]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(ProtoEncode, ProtoDecode, Debug, Clone)]
#[proto(tag_repr = "u8")]
pub enum GameEvent {
    NoRespawnBlock,
    EndRaining,
    BeginRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    ArrowHitPlayer,
    RainLevelChange,
    ThunderLevelChange,
    PlayerPufferFishSound,
    PlayElderGuardianMobAppearance,
    EnableRespawnScreen,
    LimitedCrafting,
    StartWaitingForLevelChunks,
}
