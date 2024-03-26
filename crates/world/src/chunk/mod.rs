use serverx_block::states::BlockState;
use serverx_macros::nbt;

use crate::{
    biome::Biome,
    chunk::{
        height::{Heightmap, MotionBlockingPred, WorldSurfacePred},
        section::ChunkSection,
    },
};

pub mod height;
pub mod section;

pub struct Chunk {
    sections: Vec<ChunkSection>,
    motion_blocking: Heightmap<MotionBlockingPred>,
    world_surface: Heightmap<WorldSurfacePred>,
}

impl Chunk {
    pub const WIDTH: usize = 16;

    pub fn new(height: usize) -> Self {
        let sections = height / Self::WIDTH;
        let mut chunk_sections = Vec::with_capacity(sections);
        for _ in 0..sections {
            chunk_sections.push(ChunkSection::new());
        }
        Self {
            sections: chunk_sections,
            motion_blocking: Heightmap::new(height as u64),
            world_surface: Heightmap::new(height as u64),
        }
    }

    pub fn height(&self) -> usize {
        self.sections.len() * Self::WIDTH
    }

    pub fn sections(&self) -> &[ChunkSection] {
        self.sections.as_slice()
    }

    pub fn sections_mut(&mut self) -> &mut [ChunkSection] {
        self.sections.as_mut_slice()
    }

    pub fn get_block(&self, pos: (usize, usize, usize)) -> BlockState {
        self.sections[pos.1 / Self::WIDTH].get_block((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn get_biome(&self, pos: (usize, usize, usize)) -> Biome {
        self.sections[pos.1 / Self::WIDTH].get_biome((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn set_block(&mut self, pos: (usize, usize, usize), block_state: BlockState) -> BlockState {
        self.sections[pos.1 / Self::WIDTH].set_block((pos.0, pos.1 & 0xf, pos.2), block_state)
    }

    pub fn set_biome(&mut self, pos: (usize, usize, usize), biome: Biome) -> Biome {
        self.sections[pos.1 / Self::WIDTH].set_biome((pos.0, pos.1 & 0xf, pos.2), biome)
    }

    pub fn heightmaps_tag(&self) -> serverx_nbt::Tag {
        let mut motion_blocking: Vec<i64> = Vec::with_capacity(self.motion_blocking.data().len());
        let mut world_surface: Vec<i64> = Vec::with_capacity(self.world_surface.data().len());
        motion_blocking.extend(self.motion_blocking.data().iter().map(|x| *x as i64));
        world_surface.extend(self.world_surface.data().iter().map(|x| *x as i64));
        nbt!({
            "MOTION_BLOCKING": motion_blocking,
            "WORLD_SURFACE": world_surface
        })
    }
}
