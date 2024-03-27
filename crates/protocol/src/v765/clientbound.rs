use std::{
    any::Any,
    io::{Read, Seek, Write},
    sync::Arc,
};

use serverx_common::identifier::Identifier;
use serverx_macros::{Packet, ProtoDecode, ProtoEncode};
use serverx_nbt as nbt;
use uuid::Uuid;

use crate as protocol;
use crate::{types::*, v765::types::*};

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
    #[proto(repr = "Arc<nbt::TagRoot>")]
    pub registries: Arc<nbt::Tag>,
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
#[packet(0x09, ClientBound, Configuration)]
pub struct UpdateTags {
    pub tags: Vec<RegistryTag>,
}

// Game join
// Difficulty
// Play abilities
// Selected slot
// Recipes
// EntityStatus
// Command Tree
// Unlock Recipes
// Position Look
// Server Metadata
// Player List
// World Border
// World Time Update
// Spawn Position
// Game State Change
// Update Tick Rate
// Tick Step
// Chunk Render Distance Center
// Inventory
// Entity Tracker
// Entity Attributes
// Advancement Update
//

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x0B, ClientBound, Play)]
pub struct ChangeDifficulty {
    pub difficulty: Difficulty,
    pub locked: bool,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x0C, ClientBound, Play)]
pub struct ChunkBatchFinish {
    #[proto(repr = "VarInt")]
    pub size: i32
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x0D, ClientBound, Play)]
pub struct ChunkBatchStart;

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x20, ClientBound, Play)]
pub struct ServerGameEvent {
    pub event: GameEvent,
    pub value: f32
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x25, ClientBound, Play)]
pub struct ChunkDataAndLight {
    pub x: i32,
    pub z: i32,
    #[proto(repr = "nbt::TagRoot")]
    pub heightmaps: nbt::Tag,
    pub chunk_data: Vec<u8>,
    pub block_entities: Vec<BlockEntityRecord>,
    pub chunk_lighting: ChunkLighting,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x29, ClientBound, Play)]
pub struct GameJoin {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub dimensions: Vec<Identifier>,
    #[proto(repr = "VarInt")]
    pub max_players: i32,
    #[proto(repr = "VarInt")]
    pub view_distance: i32,
    #[proto(repr = "VarInt")]
    pub sim_distance: i32,
    pub reduced_debug: bool,
    pub enable_respawn: bool,
    pub limited_crafting: bool,
    pub dimension_type: Identifier,
    pub dimension_name: Identifier,
    pub seed: i64,
    pub game_mode: GameMode,
    pub last_game_mode: LastGameMode,
    pub is_debug: bool,
    pub is_flag: bool,
    pub death_location: Option<DeathLocation>,
    #[proto(repr = "VarInt")]
    pub portal_cooldown: i32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x36, ClientBound, Play)]
pub struct PlayerAbilities {
    pub flags: u8,
    pub fly_speed: f32,
    pub fov_modifier: f32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x3E, ClientBound, Play)]
pub struct SyncPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    #[proto(repr = "VarInt")]
    pub teleport_id: i32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x52, ClientBound, Play)]
pub struct SetCenterChunk {
    #[proto(repr = "VarInt")]
    pub x: i32,
    #[proto(repr = "VarInt")]
    pub z: i32
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x54, ClientBound, Play)]
pub struct DefaultSpawnPosition {
    #[proto(repr = "Position")]
    pub location: (i32, i32, i32),
    pub angle: f32,
}

#[derive(Packet, ProtoEncode, ProtoDecode, Debug, Clone)]
#[packet(0x67, ClientBound, Play)]
pub struct StartConfiguration;
