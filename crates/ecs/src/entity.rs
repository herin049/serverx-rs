use crate::{ArchetypeId, ArchetypeIndex, Generation};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Entity {
    generation: Generation,
    archetype_id: ArchetypeId,
    archetype_index: ArchetypeIndex,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            generation: 0,
            archetype_id: 0,
            archetype_index: 0,
        }
    }
}

impl Entity {
    pub fn new(
        generation: Generation,
        archetype_id: ArchetypeId,
        archetype_index: ArchetypeIndex,
    ) -> Self {
        Self {
            generation,
            archetype_id,
            archetype_index,
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
    pub fn archetype_index(&self) -> ArchetypeIndex {
        self.archetype_index
    }
}
