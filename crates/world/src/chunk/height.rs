use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use serverx_block::states::BlockState;
use crate::chunk::Chunk;
use crate::chunk::section::{BLOCK_SECTION_SIZE, BLOCK_SECTION_WIDTH};
use serverx_common::collections::packed_vec::PackedVec;

pub trait HeightmapPred {
    fn test(block_state: BlockState) -> bool;
}

pub struct MotionBlockingPred;

impl HeightmapPred for MotionBlockingPred {
    fn test(block_state: BlockState) -> bool {
        block_state.blocks_motion()
    }
}

pub struct WorldSurfacePred;

impl HeightmapPred for WorldSurfacePred {
    fn test(block_state: BlockState) -> bool {
        !block_state.is_air()
    }
}

pub struct Heightmap<P: HeightmapPred> {
    phantom: PhantomData<P>,
    heights: PackedVec
}

impl<P: HeightmapPred> Heightmap<P> {
    pub fn new(height: u64) -> Self {
        let height_bits = (u64::BITS as u64) - ((height + 1).leading_zeros() as u64);
        Self {
            phantom: PhantomData,
            heights: PackedVec::zeros(height_bits as usize, BLOCK_SECTION_SIZE)
        }
    }

    pub fn sync(&mut self, chunk: &Chunk) {
        let mut height_range: (usize, usize) = (0, chunk.height());
        for section in chunk.sections() {
            if !section.is_empty() {
                break;
            }
            height_range.0 += BLOCK_SECTION_WIDTH;
        }
        for section in chunk.sections().iter().rev() {
            if !section.is_empty() {
                break;
            }
            height_range.1 -= BLOCK_SECTION_WIDTH;
        }
        if height_range.0 >= height_range.1 {
            height_range = (0, 0);
        }

        for x in 0..BLOCK_SECTION_WIDTH {
            for z in 0..BLOCK_SECTION_WIDTH {
                let index: usize = (x & 0xf) | ((z & 0xf) << 4);
                let mut set = false;
                for y in (height_range.0..height_range.1).rev() {
                    if P::test(chunk.get_block((x, y, z)))  {
                        self.heights.set(index, (y + 1) as u64);
                        set = true;
                        break;
                    }
                }
                if !set {
                    self.heights.set(index, 0u64);
                }
            }
        }
    }

    pub fn update(&mut self, chunk: &Chunk, pos: (usize, usize, usize), state: BlockState) {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4);
        let update_height = pos.1 + 1;
        let map_height = self.heights.get(index).unwrap() as usize;
        if update_height > map_height {
            if P::test(state)  {
                self.heights.set(index, update_height as u64);
            }
        } else if update_height == map_height {
            if !P::test(state) {
                for y in (0..pos.1) {
                    if P::test(chunk.get_block((pos.0, y, pos.2))) {
                        self.heights.set(index, (y as u64) + 1);
                        return;
                    }
                }
                self.heights.set(index, 0u64);
            }
        }
    }
}

impl<P: HeightmapPred> Debug for Heightmap<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(&self.heights, f)
    }
}