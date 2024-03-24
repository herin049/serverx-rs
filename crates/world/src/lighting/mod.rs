use crate::chunk::section::{BLOCK_SECTION_SIZE, BLOCK_SECTION_WIDTH};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LightLevel(u8);

#[derive(Clone, Debug)]
pub struct LightSection {
    occupied: u16,
    lighting: Vec<u8>
}

impl LightSection {
    pub fn new() -> Self {
        Self {
            occupied: 0,
            lighting: vec![0; BLOCK_SECTION_SIZE / 2]
        }
    }

    pub fn from_vec(lighting: Vec<u8>) -> Self {
        let mut occupied = 0;
        for l in lighting.iter() {
            if *l & 0xf != 0 {
               occupied += 1;
            }
            if *l * 0xf0 != 0 {
                occupied += 1;
            }
        }
        Self {
            occupied,
            lighting
        }
    }

    pub fn fill(&mut self, level: LightLevel) {
        self.occupied = if level.0 == 0 {
            0
        } else {
            BLOCK_SECTION_SIZE as u16
        };
        for e in self.lighting.iter_mut() {
            *e = (level.0) | (level.0 << 4);
        }
    }

    pub fn get(&self, pos: (usize, usize, usize)) -> LightLevel {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let level = (self.lighting[index] >> ((index % 2) * 4)) & 0xf;
        LightLevel(level)
    }

    pub fn set(&mut self, pos: (usize, usize, usize), level: LightLevel) -> LightLevel {
        let index: usize = (pos.0 & 0xf) | ((pos.2 & 0xf) << 4) | ((pos.1 & 0xf) << 8);
        let prev = self.get(pos);
        self.lighting[index]  = self.lighting[index] & (0xf >> ((index % 2) * 4)) | (level.0 << ((index % 2) * 4));
        prev
    }

    pub fn as_slice(&self) -> &[u8] {
        self.lighting.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.lighting.as_mut_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.occupied == 0
    }
}

#[derive(Clone, Debug)]
pub struct ChunkLighting {
    pub block_light: Vec<LightSection>,
    pub sky_light: Vec<LightSection>,
}

impl ChunkLighting {
    pub fn new(sections: usize) -> Self {
        let mut block_light = Vec::with_capacity(sections);
        let mut sky_light = Vec::with_capacity(sections);
        for _ in 0..sections {
            block_light.push(LightSection::new());
            sky_light.push(LightSection::new());
        }
        Self {
            block_light,
            sky_light
        }
    }

    pub fn height(&self) -> usize {
        self.block_light.len() * BLOCK_SECTION_WIDTH
    }

    pub fn block_lights(&self) -> &[LightSection] {
        self.block_light.as_slice()
    }

    pub fn block_lights_mut(&mut self) -> &mut [LightSection] {
        self.block_light.as_mut_slice()
    }

    pub fn sky_lights(&self) -> &[LightSection] {
        self.sky_light.as_slice()
    }

    pub fn sky_lights_mut(&mut self) -> &mut [LightSection] {
        self.sky_light.as_mut_slice()
    }

    pub fn get_block_light(&self, pos: (usize, usize, usize)) -> LightLevel {
        self.block_light[pos.1 / BLOCK_SECTION_WIDTH].get((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn set_block_light(&mut self, pos: (usize, usize, usize), level: LightLevel) -> LightLevel {
        self.block_light[pos.1 / BLOCK_SECTION_WIDTH].set((pos.0, pos.1 & 0xf, pos.2), level)
    }

    pub fn get_sky_light(&self, pos: (usize, usize, usize)) -> LightLevel {
        self.sky_light[pos.1 / BLOCK_SECTION_WIDTH].get((pos.0, pos.1 & 0xf, pos.2))
    }

    pub fn set_sky_light(&mut self, pos: (usize, usize, usize), level: LightLevel) -> LightLevel {
        self.sky_light[pos.1 / BLOCK_SECTION_WIDTH].set((pos.0, pos.1 & 0xf, pos.2), level)
    }
}

