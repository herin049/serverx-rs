use std::mem::MaybeUninit;

use slab::Slab;

use crate::ecs::{entity::Entity, storage::component::ComponentStorage, tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple}, Index, ArchetypeId, Generation};

pub struct ArchetypeStorage {
    pub archetype_id: ArchetypeId,
    pub components: Vec<MaybeUninit<Box<dyn ComponentStorage>>>,
    pub entities: Vec<Entity>,
    pub entity_lookup: Slab<Index>,
    pub generation: Generation
}

impl ArchetypeStorage {
    pub fn new<T: ComponentTuple>(archetype_id: ArchetypeId) -> Self {
        let mut components = Vec::new();
        for component_id in T::COMPONENT_SET.iter() {
            if (component_id as usize) >= components.len() {
                let diff = (component_id as usize + 1) - components.len();
                components.reserve(diff);
                unsafe {
                    components.set_len(component_id as usize + 1);
                }
            }
        }
        let mut storage = ArchetypeStorage {
            archetype_id,
            components,
            entities: Vec::new(),
            entity_lookup: Slab::new(),
            generation: 0
        };
        unsafe {
            T::init_storage(&mut storage);
        }
        storage
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub unsafe fn get_components<'a, T: ComponentRefTuple<'a>>(&'a self, entity: Entity) -> Option<T> {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            if *self.entities.get_unchecked(*index as usize) == entity {
                return Some(T::get(self, *index));
            }
        }
        None
    }

    pub unsafe fn get_components_mut<'a, T: ComponentBorrowTuple<'a>>(&'a self, entity: Entity) -> Option<T> {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            if *self.entities.get_unchecked(*index as usize) == entity {
                return Some(T::get(self, *index));
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
        T::push(values, self);
        Entity::new(self.archetype_id, archetype_index, generation)
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            unsafe {
                self.components
                    .iter_mut()
                    .for_each(|component| {
                        component.assume_init_mut().swap_remove(*index)
                    });
            }

            let updated_index = self.entities.last().unwrap().archetype_index();
            self.entities.swap_remove(*index as usize);
            if (updated_index as usize) < self.entities.len() {
                updated_index
            }

        }
        true
    }
}
