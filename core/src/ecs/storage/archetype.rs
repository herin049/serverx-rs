use std::{
    cmp,
    fmt::{Debug, Formatter},
    mem::MaybeUninit,
    ops::Range,
};

use itertools::Itertools;
use slab::Slab;

use crate::ecs::{
    component::ComponentSet,
    entity::{DebugEntity, Entity},
    storage::component::ComponentStorage,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    ArchetypeId, Generation, Index,
};

pub struct ArchetypeStorage {
    pub archetype_id: ArchetypeId,
    pub component_set: ComponentSet,
    pub components: Vec<MaybeUninit<Box<dyn ComponentStorage>>>,
    pub entities: Vec<Entity>,
    pub entity_lookup: Slab<Index>,
    pub generation: Generation,
}

impl ArchetypeStorage {
    pub fn new<T: ComponentTuple>(archetype_id: ArchetypeId) -> Self {
        let mut components = Vec::new();
        let max_component_id = T::COMPONENT_SET.iter().max().unwrap();
        components.reserve(max_component_id as usize + 1);
        unsafe {
            components.set_len(max_component_id as usize + 1);
        }
        let mut storage = ArchetypeStorage {
            archetype_id,
            component_set: T::COMPONENT_SET,
            components,
            entities: Vec::new(),
            entity_lookup: Slab::new(),
            generation: 0,
        };
        unsafe {
            T::init_archetype_storage(&mut storage);
        }
        storage
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub unsafe fn get_components<'a, T: ComponentRefTuple<'a>>(
        &'a self,
        entity: Entity,
    ) -> Option<T> {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            if *self.entities.get_unchecked(*index as usize) == entity {
                return Some(T::get_components(self, *index));
            }
        }
        None
    }

    pub unsafe fn get_components_mut<'a, T: ComponentBorrowTuple<'a>>(
        &'a self,
        entity: Entity,
    ) -> Option<T> {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            if *self.entities.get_unchecked(*index as usize) == entity {
                return Some(T::get_components(self, *index));
            }
        }
        None
    }

    pub unsafe fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let vacant_entry = self.entity_lookup.vacant_entry();
        let archetype_index = vacant_entry.key() as Index;
        let generation = self.generation;
        self.generation = self.generation.wrapping_add(1);
        vacant_entry.insert(self.entities.len() as Index);
        let entity = Entity::new(self.archetype_id, archetype_index, generation);
        self.entities.push(entity);
        T::push_components(values, self);
        entity
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        let index = if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            unsafe {
                if *self.entities.get_unchecked(*index as usize) != entity {
                    return false;
                }
            }
            *index
        } else {
            return false;
        };
        if self.entities[index as usize] != entity {
            return false;
        }
        unsafe {
            self.components
                .iter_mut()
                .for_each(|component| component.assume_init_mut().swap_remove_component(index));
        }

        let updated_index = self.entities.last().unwrap().archetype_index();
        self.entities.swap_remove(index as usize);
        if let Some(archetype_index) = self.entity_lookup.get_mut(updated_index as usize) {
            *archetype_index = index;
        }
        self.entity_lookup.remove(entity.archetype_index() as usize);
        true
    }

    pub fn chunks(&self, chunk_size: usize) -> ArchetypeChunkIter {
        ArchetypeChunkIter {
            archetype: self,
            curr: 0,
            chunk_size,
        }
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list_fmt = f.debug_list();
        for entity in self.entities.iter() {
            list_fmt.entry(&DebugEntity {
                archetype: self,
                entity,
            });
        }
        list_fmt.finish()
    }
}

pub struct ArchetypeChunkIter<'a> {
    pub archetype: &'a ArchetypeStorage,
    pub curr: Index,
    pub chunk_size: usize,
}

impl<'a> Iterator for ArchetypeChunkIter<'a> {
    type Item = ArchetypeChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.curr as usize) < self.archetype.len() {
            let next = cmp::min(
                self.curr + (self.chunk_size as Index),
                self.archetype.len() as Index,
            );
            let range = Range {
                start: self.curr,
                end: next,
            };
            self.curr = next;
            Some(ArchetypeChunk {
                archetype: &self.archetype,
                range,
            })
        } else {
            None
        }
    }
}

pub struct ArchetypeChunk<'a> {
    pub archetype: &'a ArchetypeStorage,
    pub range: Range<Index>,
}

impl<'a> ArchetypeChunk<'a> {
    pub fn entities(&self) -> &[Entity] {
        &self.archetype.entities[(self.range.start as usize)..(self.range.end as usize)]
    }
}

impl<'a> Debug for ArchetypeChunk<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list_fmt = f.debug_list();
        for entity in self.entities() {
            list_fmt.entry(&DebugEntity {
                archetype: &self.archetype,
                entity,
            });
        }
        list_fmt.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{component::Component, storage::archetype::ArchetypeStorage, ComponentId};

    #[derive(Debug)]
    pub struct Position(i32, i32, i32);
    #[derive(Debug)]
    pub struct Velocity(i32, i32, i32);

    unsafe impl Component for Position {
        const ID: ComponentId = 0;
    }

    unsafe impl Component for Velocity {
        const ID: ComponentId = 1;
    }

    #[test]
    fn test() {
        unsafe {
            let mut arch = ArchetypeStorage::new::<(Position, Velocity)>(0);
            println!("{:#?}", arch);
            let e1 = arch.push((Position(1, 1, 1), Velocity(1, 1, 1)));
            let e2 = arch.push((Position(2, 2, 2), Velocity(2, 2, 2)));
            let e3 = arch.push((Position(3, 3, 3), Velocity(3, 3, 3)));
            println!("{:#?}", arch);
            arch.remove(e2);
            if let Some((p, v)) = arch.get_components_mut::<(&mut Position, &mut Velocity)>(e3) {
                p.0 = 100;
                v.0 *= 100;
            }
            println!("{:#?}", arch);
            let e4 = arch.push((Position(4, 4, 4), Velocity(4, 4, 4)));
            println!("{:#?}", arch);
            arch.remove(e1);
            arch.remove(e3);
            arch.remove(e4);
            println!("{:#?}", arch);
        }
    }

    #[test]
    fn test2() {
        unsafe {
            let mut arch = ArchetypeStorage::new::<(Position, Velocity)>(0);
            for i in 0..20 {
                arch.push((Position(i, i, i), Velocity(i, i, i)));
            }
            println!("{:?}", arch);
            for (index, chunk) in arch.chunks(6).enumerate() {
                println!("chunk {} = {:#?}", index, chunk);
            }
        }
    }
}
