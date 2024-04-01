use std::marker::PhantomData;

use crate::{
    entity::Entity,
    registry::UnsafeRegistryCell,
    tuple::{
        component::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
        value::ValueTuple,
    },
    util,
};
use crate::archetype::{ArchetypeId, ArchetypeIdx};

pub struct Accessor<'a, 'b, L: ComponentBorrowTuple<'b>, G: ComponentBorrowTuple<'b>> {
    phantom: PhantomData<&'b (L, G)>,
    registry: UnsafeRegistryCell<'a>,
    pub(crate) iter_pos: (ArchetypeId, ArchetypeIdx),
}

impl<'a, 'b, L: ComponentBorrowTuple<'b>, G: ComponentBorrowTuple<'b>> Accessor<'a, 'b, L, G> {
    pub fn new<'r>(registry: UnsafeRegistryCell<'r>) -> Self
    where
        'r: 'a,
    {
        Self {
            phantom: PhantomData,
            registry,
            iter_pos: (ArchetypeId::MAX, ArchetypeIdx::MAX)
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.registry.contains(entity)
    }

    pub fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        self.registry.has::<T>(entity)
    }

    pub fn get<'c, T: ComponentRefTuple<'c>>(&self, entity: Entity) -> Option<T>
    where
        'a: 'c,
    {
        let type_ids = T::ValueType::type_ids();
        if entity.archetype_id() == self.iter_pos.0 {
            if self.registry.archetypes()[self.iter_pos.0 as usize].index_of(entity) == Some(self.iter_pos.1 as usize) {
                if !util::disjoint(type_ids.as_ref(), L::WriteType::type_ids().as_ref())
                {
                    panic!("invalid get");
                }
            }
        }
        if !util::subset(type_ids.as_ref(), G::ValueType::type_ids().as_ref()) {
            panic!("invalid get");
        }
        unsafe { self.registry.get::<'c, T>(entity) }
    }

    pub fn get_mut<'c, T: ComponentBorrowTuple<'c>>(&mut self, entity: Entity) -> Option<T>
    where
        'a: 'c,
    {
        let type_ids = T::ValueType::type_ids();
        if entity.archetype_id() == self.iter_pos.0 {
            if self.registry.archetypes()[self.iter_pos.0 as usize].index_of(entity) == Some(self.iter_pos.1 as usize) {
                if !util::disjoint(type_ids.as_ref(), L::WriteType::type_ids().as_ref())
                {
                    panic!("invalid get");
                }
            }
        }
        if !util::subset(
            T::ReadType::type_ids().as_ref(),
            G::ValueType::type_ids().as_ref(),
        ) || !util::subset(
            T::WriteType::type_ids().as_ref(),
            G::WriteType::type_ids().as_ref(),
        ) {
            panic!("invalid get mut");
        }
        unsafe { self.registry.get_mut::<'c, T>(entity) }
    }
}
