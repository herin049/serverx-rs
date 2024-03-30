use serverx_block::blocks::Block;
use serverx_common::{collections::bit_vec::BitVec, identifier};
use serverx_game::chunk::generators::{flat::FlatGeneratorBuilder, ChunkGenerator};
use serverx_macros::identifier;
use serverx_protocol::{
    chunk::encode_chunk,
    v765::{
        clientbound::{
            ChangeDifficulty, ChunkBatchFinish, ChunkBatchStart, ChunkDataAndLight,
            DefaultSpawnPosition, FeatureFlags, GameJoin, PlayerAbilities, RegistryData,
            ServerFinishConfiguration, ServerGameEvent, SetCenterChunk, SyncPlayerPosition,
            UpdateTags,
        },
        types::{ChunkLighting, Difficulty, GameEvent, GameMode, LastGameMode},
    },
};
use tracing::instrument;

use crate::{
    client::{status::ClientStatus, Client},
    network::brand::make_server_brand_message,
    server::Server,
};

#[instrument(skip_all)]
pub fn update_client(client: &mut Client, server: &mut Server) {
    match client.status {
        ClientStatus::Init => {
            let server_brand = make_server_brand_message(&identifier!("vanilla"));
            tracing::trace!(?server_brand, "sending server brand message");
            let _ = client.outgoing.send(Box::new(server_brand));
            let feature_flags = FeatureFlags {
                flags: vec![identifier!("vanilla")],
            };
            tracing::trace!(?feature_flags, "sending feature flags packet");
            let _ = client.outgoing.send(Box::new(feature_flags));
            let registry_data = RegistryData {
                registries: server.resources.registry_data.clone(),
            };
            tracing::trace!("sending registry data packet");
            let _ = client.outgoing.send(Box::new(registry_data));
            let update_tags = UpdateTags { tags: vec![] };
            tracing::trace!(?update_tags, "sending update tags packet");
            let _ = client.outgoing.send(Box::new(update_tags));
            let finish_config = ServerFinishConfiguration;
            tracing::trace!("sending finish configuration packet");
            let _ = client.outgoing.send(Box::new(finish_config));

            // TEST
            let _ = client.outgoing.send(Box::new(GameJoin {
                entity_id: 0,
                is_hardcore: false,
                dimensions: vec![
                    identifier!("overworld"),
                    identifier!("the_nether"),
                    identifier!("the_end"),
                ],
                max_players: 100,
                view_distance: 6,
                sim_distance: 6,
                reduced_debug: false,
                enable_respawn: false,
                limited_crafting: false,
                dimension_type: identifier!("overworld"),
                dimension_name: identifier!("overworld"),
                seed: 0,
                game_mode: GameMode::Survival,
                last_game_mode: LastGameMode::Undefined,
                is_debug: false,
                is_flag: false,
                death_location: None,
                portal_cooldown: 0,
            }));
            let _ = client.outgoing.send(Box::new(ChangeDifficulty {
                difficulty: Difficulty::Hard,
                locked: false,
            }));
            let _ = client.outgoing.send(Box::new(PlayerAbilities {
                flags: 0,
                fly_speed: 0.5,
                fov_modifier: 0.1,
            }));

            let _ = client
                .outgoing
                .send(Box::new(SetCenterChunk { x: 0, z: 0 }));

            let generator = FlatGeneratorBuilder::new(384)
                .layer(Block::IronBlock, 64)
                .build();
            let chunk = generator.generate((0, 0));
            // println!("{:?}", chunk.sections().get(4).unwrap().blocks);
            let _ = client.outgoing.send(Box::new(ServerGameEvent {
                event: GameEvent::StartWaitingForLevelChunks,
                value: 0.0,
            }));
            let _ = client.outgoing.send(Box::new(ChunkBatchStart));
            if let Ok(encoded) = encode_chunk(&chunk) {
                let heightmaps = chunk.heightmaps_tag();
                for i in -6..=6 {
                    for j in -6..=6 {
                        let _ = client.outgoing.send(Box::new(ChunkDataAndLight {
                            x: i,
                            z: j,
                            heightmaps: heightmaps.clone(),
                            chunk_data: encoded.clone(),
                            block_entities: vec![],
                            chunk_lighting: ChunkLighting {
                                sky_light_mask: BitVec::zeros(chunk.sections().len()),
                                block_light_mask: BitVec::zeros(chunk.sections().len()),
                                empty_sky_light_mask: BitVec::ones(chunk.sections().len()),
                                empty_block_light_mask: BitVec::ones(chunk.sections().len()),
                                sky_light_sections: vec![],
                                block_light_sections: vec![],
                            },
                        }));
                    }
                }
            }
            let _ = client
                .outgoing
                .send(Box::new(ChunkBatchFinish { size: 49 }));
            let _ = client.outgoing.send(Box::new(DefaultSpawnPosition {
                location: (0, 10, 0),
                angle: 0.0,
            }));

            let _ = client.outgoing.send(Box::new(SyncPlayerPosition {
                x: 0.0,
                y: 10.0,
                z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                flags: 0,
                teleport_id: 0,
            }));

            client.status = ClientStatus::Connecting;
        }
        ClientStatus::Connecting => {
            client.status = ClientStatus::Connected;
        }
        ClientStatus::Connected => {
            if client.incoming.is_disconnected() || client.outgoing.is_disconnected() {
                client.status = ClientStatus::Disconnecting;
            }
        }
        ClientStatus::Disconnecting => {
            tracing::debug!(profile = ?client.profile, "client disconnected");
            client.status = ClientStatus::Disconnected;
        }
        ClientStatus::Disconnected => {}
    }
}
