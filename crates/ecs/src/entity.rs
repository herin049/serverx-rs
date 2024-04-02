use crate::archetype::{ArchetypeId, ArchetypeIdx, Generation};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Entity {
    generation: Generation,
    archetype_id: ArchetypeId,
    archetype_idx: ArchetypeIdx,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            generation: 0,
            archetype_id: 0,
            archetype_idx: 0,
        }
    }
}

impl Entity {
    pub fn new(
        generation: Generation,
        archetype_id: ArchetypeId,
        archetype_idx: ArchetypeIdx,
    ) -> Self {
        Self {
            generation,
            archetype_id,
            archetype_idx,
        }
    }

    #[inline(always)]
    pub fn generation(&self) -> Generation {
        self.generation
    }

    #[inline(always)]
    pub fn archetype_id(&self) -> ArchetypeId {
        self.archetype_id
    }

    #[inline(always)]
    pub fn archetype_idx(&self) -> ArchetypeIdx {
        self.archetype_idx
    }
}
