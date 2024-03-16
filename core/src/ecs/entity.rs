use std::fmt::{Debug, Display, Formatter};

use crate::ecs::{ArchetypeId, Generation, Index};

#[derive(PartialEq, Eq)]
pub struct Entity {
    archetype_id: ArchetypeId,
    archetype_index: Index,
    generation: Generation
}

impl Entity {
    pub fn new(archetype_id: ArchetypeId, archetype_index: Index, generation: Generation) -> Self {
        Self {
            archetype_id,
            archetype_index,
            generation
        }
    }

    pub fn archetype_id(&self) -> ArchetypeId {
        self.archetype_id
    }

    pub fn archetype_index(&self) -> Index {
        self.archetype_index
    }

    pub fn generation(&self) -> Generation {
        self.generation
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity(archetype_id={}, archetype_index={}, generation={})",
            self.archetype_id(),
            self.archetype_index(),
            self.generation()
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
