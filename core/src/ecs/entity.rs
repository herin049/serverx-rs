use std::fmt::{Debug, Display, Formatter};

use crate::ecs::{ArchetypeId, Generation, Index};

#[derive(PartialEq, Eq)]
pub struct Entity(u64);

impl Entity {
    pub fn new(index: Index, generation: Generation, archetype_id: ArchetypeId) -> Self {
        Self(
            ((index as u32) as u64)
                | (((generation as u16) as u64) << 32)
                | (((archetype_id as u16) as u64) << 48),
        )
    }

    pub fn index(&self) -> Index {
        (self.0 as u32) as Index
    }

    pub fn generation(&self) -> Generation {
        ((self.0 >> 32) as u16) as Generation
    }

    pub fn archetype(&self) -> ArchetypeId {
        ((self.0 >> 48) as u16) as ArchetypeId
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity(index={}, generation={}, archetype={})",
            self.index(),
            self.generation(),
            self.archetype()
        )
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::ecs::entity::Entity;

    #[test]
    pub fn test() {
        let a = Entity::new(123, 456, 789);
        println!("{}", a);
    }
}
