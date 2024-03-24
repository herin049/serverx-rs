use crate::biome::Biome;
use serverx_block::states::BlockState;
use crate::chunk::section::{BLOCK_SECTION_WIDTH, ChunkSection};

pub mod section;
pub mod height;

pub struct Chunk {
    sections: Vec<ChunkSection>,
}

impl Chunk {
    pub fn new(sections: usize) -> Self {
        let mut chunk_sections = Vec::with_capacity(sections);
        for _ in 0..sections {
            chunk_sections.push(ChunkSection::new());
        }
        Self {
            sections: chunk_sections
        }
    }

    pub fn height(&self) -> usize {
        self.sections.len() * BLOCK_SECTION_WIDTH
    }

    pub fn sections(&self) -> &[ChunkSection] {
        self.sections.as_slice()
    }

    pub fn sections_mut(&mut self) -> &mut [ChunkSection] {
        self.sections.as_mut_slice()
    }

    pub fn get_block(&self, pos: (usize, usize, usize)) -> BlockState {
        self.sections[pos.1 / BLOCK_SECTION_WIDTH].get_block((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn get_biome(&self, pos: (usize, usize, usize)) -> Biome {
        self.sections[pos.1 / BLOCK_SECTION_WIDTH].get_biome((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn set_block(&mut self, pos: (usize, usize, usize), block_state: BlockState) -> BlockState {
        self.sections[pos.1 / BLOCK_SECTION_WIDTH].set_block((pos.0, pos.1 & 0xf, pos.2), block_state)
    }

    pub fn set_biome(&mut self, pos: (usize, usize, usize), biome: Biome) -> Biome {
        self.sections[pos.1 / BLOCK_SECTION_WIDTH].set_biome((pos.0, pos.1 & 0xf, pos.2), biome)
    }
}