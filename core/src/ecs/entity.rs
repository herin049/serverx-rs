use std::fmt::{Debug, Display, Formatter};

use log::debug;

use crate::ecs::{storage::archetype::ArchetypeStorage, ArchetypeId, Generation, Index};

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Entity {
    archetype_id: ArchetypeId,
    archetype_index: Index,
    generation: Generation,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            archetype_id: 0,
            archetype_index: 0,
            generation: 0,
        }
    }
}

impl Entity {
    pub fn new(archetype_id: ArchetypeId, archetype_index: Index, generation: Generation) -> Self {
        Self {
            archetype_id,
            archetype_index,
            generation,
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

pub struct DebugEntity<'a> {
    pub archetype: &'a ArchetypeStorage,
    pub entity: &'a Entity,
}

impl<'a> Debug for DebugEntity<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_tuple = f.debug_tuple("DebugEntity");
        let storage_index = unsafe {
            *self
                .archetype
                .entity_lookup
                .get(self.entity.archetype_index as usize)
                .unwrap()
        };
        debug_tuple.field(self.entity);
        for component_id in self.archetype.component_set.iter() {
            unsafe {
                debug_tuple.field(
                    self.archetype.components[component_id as usize]
                        .assume_init_ref()
                        .get_dyn_component(storage_index),
                );
            }
        }
        debug_tuple.finish()
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
