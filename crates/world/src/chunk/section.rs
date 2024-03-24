use crate::{
    biome,
    biome::Biome,
};

use serverx_block::states;
use serverx_block::states::BlockState;
use serverx_common::collections::pallet::{PalletContainer, PalletOpts};

pub const BLOCK_SECTION_WIDTH: usize = 16;
pub const BLOCK_SECTION_SIZE: usize =
    BLOCK_SECTION_WIDTH * BLOCK_SECTION_WIDTH * BLOCK_SECTION_WIDTH;
pub const BLOCK_BITS: usize =
    (u64::BITS - ((states::STATE_COUNT - 1) as u64).leading_zeros()) as usize;
pub const BIOME_SECTION_WIDTH: usize = 4;
pub const BIOME_SECTION_SIZE: usize =
    BIOME_SECTION_WIDTH * BIOME_SECTION_WIDTH * BIOME_SECTION_WIDTH;
pub const BIOME_BITS: usize =
    (u64::BITS - ((biome::BIOME_COUNT - 1) as u64).leading_zeros()) as usize;

pub struct ChunkSection {
    pub blocks: PalletContainer,
    pub biomes: PalletContainer,
    pub occupied: u16,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            blocks: PalletContainer::single(
                PalletOpts::new(BLOCK_BITS as u8, (4, 8)),
                BLOCK_SECTION_SIZE,
                BlockState::default().id(),
            ),
            biomes: PalletContainer::single(
                PalletOpts::new(BIOME_BITS as u8, (1, 3)),
                BIOME_SECTION_SIZE,
                Biome::default().id(),
            ),
            occupied: 0,
        }
    }

    pub fn fill_blocks(&mut self, state: BlockState) {
        self.occupied = if state == BlockState::default() {
            0
        } else {
            BLOCK_SECTION_SIZE as u16
        };
        self.blocks.fill(state.id());
    }

    pub fn fill_biomes(&mut self, biome: Biome) {
        self.biomes.fill(biome.id());
    }

    pub fn get_block(&self, pos: (usize, usize, usize)) -> BlockState {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let id = self.blocks.get(index).unwrap();
        BlockState::try_from(id).unwrap_or_default()
    }

    pub fn get_biome(&self, pos: (usize, usize, usize)) -> Biome {
        let index: usize =
            ((pos.0 >> 2) & 0x3) | (((pos.2 >> 2) << 2) & 0x3) | (((pos.1 >> 2) << 4) & 0x3);
        let id = self.biomes.get(index).unwrap();
        Biome::try_from(id).unwrap_or_default()
    }

    pub fn set_block(&mut self, pos: (usize, usize, usize), state: BlockState) -> BlockState {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let prev =
            BlockState::try_from(self.blocks.set(index, state.id()).unwrap()).unwrap_or_default();
        if prev != state {
            if prev == BlockState::default() {
                self.occupied += 1;
            } else {
                self.occupied -= 1;
            }
        }
        prev
    }

    pub fn set_biome(&mut self, pos: (usize, usize, usize), biome: Biome) -> Biome {
        let index: usize =
            ((pos.0 >> 2) & 0x3) | (((pos.2 >> 2) << 2) & 0x3) | (((pos.1 >> 2) << 4) & 0x3);
        Biome::try_from(self.biomes.set(index, biome.id()).unwrap()).unwrap_or_default()
    }

    pub fn block_pallet(&self) -> &PalletContainer {
        &self.blocks
    }

    pub fn occupied(&self) -> u16 {
        self.occupied
    }

    pub fn is_empty(&self) -> bool {
        self.occupied != 0
    }
}
