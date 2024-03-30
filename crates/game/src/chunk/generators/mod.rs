pub mod flat;

use serverx_world::{chunk::Chunk, position::ChunkPosition};

pub trait ChunkGenerator {
    fn generate(&self, position: ChunkPosition) -> Chunk;
}
