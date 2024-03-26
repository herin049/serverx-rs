mod flat;

use serverx_world::chunk::Chunk;

use crate::chunk::store::ChunkPosition;

pub trait ChunkGenerator {
    fn generate(&self, chunk_position: ChunkPosition) -> Chunk;
}
