use std::fmt::{Debug, Formatter};
use serverx_block::{states, states::BlockState};
use serverx_block::blocks::Block;
use serverx_common::collections::pallet::{PalletContainer, PalletOpts};

use crate::{biome, biome::Biome};

pub struct BlockPallet {
    pub pallet: PalletContainer,
}

impl BlockPallet {
    pub const INDIRECT_RANGE: (usize, usize) = (4, 8);
    pub const PALLET_OPTS: PalletOpts = PalletOpts {
        repr_bits: BlockState::BITS as u8,
        indirect_range: (Self::INDIRECT_RANGE.0 as u8, Self::INDIRECT_RANGE.1 as u8),
    };
    pub const SIZE: usize = Self::WIDTH * Self::WIDTH * Self::WIDTH;
    pub const WIDTH: usize = 16;

    pub fn new() -> Self {
        Self {
            pallet: PalletContainer::single(
                Self::PALLET_OPTS,
                Self::SIZE,
                BlockState::default().id(),
            ),
        }
    }
}

impl Debug for BlockPallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        for i in 0..Self::SIZE {
            let b: Block = BlockState::try_from(self.pallet.get(i).unwrap()).unwrap_or_default().into();
            debug_list.entry(&b);
        }
        debug_list.finish()
    }
}

pub struct BiomePallet {
    pub pallet: PalletContainer,
}

impl BiomePallet {
    pub const INDIRECT_RANGE: (usize, usize) = (1, 3);
    pub const PALLET_OPTS: PalletOpts = PalletOpts {
        repr_bits: Biome::BITS as u8,
        indirect_range: (Self::INDIRECT_RANGE.0 as u8, Self::INDIRECT_RANGE.1 as u8),
    };
    pub const SIZE: usize = Self::WIDTH * Self::WIDTH * Self::WIDTH;
    pub const WIDTH: usize = 4;

    pub fn new() -> Self {
        Self {
            pallet: PalletContainer::single(Self::PALLET_OPTS, Self::SIZE, Biome::default().id()),
        }
    }
}

impl Debug for BiomePallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        for i in 0..Self::SIZE {
            debug_list.entry(&Biome::try_from(self.pallet.get(i).unwrap()).unwrap_or_default());
        }
        debug_list.finish()
    }
}

pub struct ChunkSection {
    pub blocks: BlockPallet,
    pub biomes: BiomePallet,
    pub occupied: u16,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            blocks: BlockPallet::new(),
            biomes: BiomePallet::new(),
            occupied: 0,
        }
    }

    pub fn fill_blocks(&mut self, state: BlockState) {
        self.occupied = if state == BlockState::default() {
            0
        } else {
            BlockPallet::SIZE as u16
        };
        self.blocks.pallet.fill(state.id());
    }

    pub fn fill_biomes(&mut self, biome: Biome) {
        self.biomes.pallet.fill(biome.id());
    }

    pub fn get_block(&self, pos: (usize, usize, usize)) -> BlockState {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let id = self.blocks.pallet.get(index).unwrap();
        BlockState::try_from(id).unwrap_or_default()
    }

    pub fn get_biome(&self, pos: (usize, usize, usize)) -> Biome {
        let index: usize =
            ((pos.0 >> 2) & 0x3) | (((pos.2 >> 2) << 2) & 0x3) | (((pos.1 >> 2) << 4) & 0x3);
        let id = self.biomes.pallet.get(index).unwrap();
        Biome::try_from(id).unwrap_or_default()
    }

    pub fn set_block(&mut self, pos: (usize, usize, usize), state: BlockState) -> BlockState {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let prev = BlockState::try_from(self.blocks.pallet.set(index, state.id()).unwrap())
            .unwrap_or_default();
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
        Biome::try_from(self.biomes.pallet.set(index, biome.id()).unwrap()).unwrap_or_default()
    }

    pub fn block_pallet(&self) -> &PalletContainer {
        &self.blocks.pallet
    }

    pub fn biome_pallet(&self) -> &PalletContainer {
        &self.biomes.pallet
    }

    pub fn occupied(&self) -> u16 {
        self.occupied
    }

    pub fn is_empty(&self) -> bool {
        self.occupied != 0
    }
}
