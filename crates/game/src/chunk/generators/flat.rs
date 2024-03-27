use serverx_block::blocks::Block;
use serverx_world::chunk::Chunk;

use crate::chunk::{generators::ChunkGenerator, store::ChunkPosition};

pub struct FlatGeneratorBuilder {
    layers: Vec<Block>,
    world_height: usize,
}

impl FlatGeneratorBuilder {
    pub fn new(world_height: usize) -> Self {
        Self {
            layers: Vec::new(),
            world_height,
        }
    }

    pub fn layer(mut self, block: Block, count: usize) -> Self {
        for _ in 0..count {
            self.layers.push(block);
        }
        Self {
            layers: self.layers,
            world_height: self.world_height,
        }
    }

    pub fn build(mut self) -> FlatGenerator {
        while self.layers.len() > self.world_height {
            self.layers.pop();
        }
        FlatGenerator {
            layers: self.layers,
            world_height: self.world_height,
        }
    }
}

pub struct FlatGenerator {
    layers: Vec<Block>,
    world_height: usize,
}

impl ChunkGenerator for FlatGenerator {
    fn generate(&self, chunk_position: ChunkPosition) -> Chunk {
        let mut chunk = Chunk::new(self.world_height);
        for (y, b) in self.layers.iter().enumerate().rev() {
            for x in 0..Chunk::WIDTH {
                for z in 0..Chunk::WIDTH {
                    chunk.set_block((x, y, z), (*b).into());
                }
            }
        }
        chunk
    }
}
