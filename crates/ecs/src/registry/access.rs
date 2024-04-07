use std::marker::PhantomData;

use crate::{
    archetype::{ArchetypeId, ArchetypeIdx},
    entity::Entity,
    registry::UnsafeRegistryCell,
    tuple::{
        component::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
        value::ValueTuple,
    },
    util,
};

pub trait Accessor {
    fn contains(&self, entity: Entity) -> bool;
    fn has<T: ComponentTuple>(&self, entity: Entity) -> bool;
    fn get<'a, 'b, T: ComponentRefTuple<'b>>(&'a self, entity: Entity) -> Option<T>
    where
        'a: 'b;
    fn get_mut<'a, 'b, T: ComponentBorrowTuple<'b>>(&'a mut self, entity: Entity) -> Option<T>
    where
        'a: 'b;
}

pub struct IterAccessor<'a, L: ComponentBorrowTuple<'static>, G: ComponentBorrowTuple<'static>> {
    phantom: PhantomData<(L, G)>,
    registry: UnsafeRegistryCell<'a>,
    pub(crate) iter_pos: (ArchetypeId, ArchetypeIdx),
}

impl<'a, L: ComponentBorrowTuple<'static>, G: ComponentBorrowTuple<'static>>
    IterAccessor<'a, L, G>
{
    pub fn new<'r>(registry: UnsafeRegistryCell<'r>) -> Self
    where
        'r: 'a,
    {
        Self {
            phantom: PhantomData,
            registry,
            iter_pos: (ArchetypeId::MAX, ArchetypeIdx::MAX),
        }
    }
}

impl<'a, L: ComponentBorrowTuple<'static>, G: ComponentBorrowTuple<'static>> Accessor
    for IterAccessor<'a, L, G>
{
    fn contains(&self, entity: Entity) -> bool {
        self.registry.contains(entity)
    }

    fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        self.registry.has::<T>(entity)
    }

    fn get<'b, 'c, T: ComponentRefTuple<'c>>(&'b self, entity: Entity) -> Option<T>
    where
        'b: 'c,
    {
        let type_ids = T::ValueType::type_ids();
        if entity.archetype_id() == self.iter_pos.0 {
            if self.registry.archetypes()[self.iter_pos.0 as usize].index_of(entity)
                == Some(self.iter_pos.1 as usize)
            {
                if !util::disjoint(type_ids.as_ref(), L::WriteType::type_ids().as_ref()) {
                    panic!("invalid get");
                }
            }
        }
        if !util::subset(type_ids.as_ref(), G::ValueType::type_ids().as_ref()) {
            panic!("invalid get");
        }
        unsafe { self.registry.get::<'c, T>(entity) }
    }

    fn get_mut<'b, 'c, T: ComponentBorrowTuple<'c>>(&'b mut self, entity: Entity) -> Option<T>
    where
        'b: 'c,
    {
        let type_ids = T::ValueType::type_ids();
        if entity.archetype_id() == self.iter_pos.0 {
            if self.registry.archetypes()[self.iter_pos.0 as usize].index_of(entity)
                == Some(self.iter_pos.1 as usize)
            {
                if !util::disjoint(type_ids.as_ref(), L::WriteType::type_ids().as_ref()) {
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
