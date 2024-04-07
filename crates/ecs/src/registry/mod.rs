use core::fmt::{Debug, Formatter};
use std::any::TypeId;

use crate::{
    archetype::{Archetype, ArchetypeId, DebugArchetypeEntry, UnsafeArchetypeCell},
    entity::Entity,
    message::Messages,
    tuple::{
        borrow::{BorrowTuple, BorrowType},
        component::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
        value::ValueTuple,
    },
    util,
};

pub mod access;

pub struct Registry {
    archetypes: Vec<Archetype>,
    archetype_lookup: Vec<(Box<[TypeId]>, ArchetypeId)>,
    messages: Messages,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            archetype_lookup: Vec::new(),
            messages: Messages::new(),
        }
    }

    pub fn messages(&self) -> &Messages {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut Messages {
        &mut self.messages
    }

    pub fn archetypes(&self) -> &[Archetype] {
        self.archetypes.as_slice()
    }

    pub fn archetypes_mut(&mut self) -> &mut [Archetype] {
        self.archetypes.as_mut_slice()
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let mut tys = T::type_ids();
        util::insertion_sort(tys.as_mut());
        let search = self
            .archetype_lookup
            .binary_search_by_key(&tys.as_ref(), |x| x.0.as_ref());
        match search {
            Ok(i) => unsafe {
                let archetype_id = self.archetype_lookup.get_unchecked(i).1;
                self.archetypes
                    .get_unchecked_mut(archetype_id as usize)
                    .push(values)
            },
            Err(i) => {
                util::assert_no_alias(tys.as_ref(), "aliasing in component tuple");
                let archetype_id = self.archetypes.len() as ArchetypeId;
                self.archetypes.push(Archetype::new::<T>(archetype_id));
                self.archetype_lookup.insert(i, (tys.into(), archetype_id));
                unsafe {
                    self.archetypes
                        .get_unchecked_mut(archetype_id as usize)
                        .push(values)
                }
            }
        }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        self.archetypes
            .get_mut(entity.archetype_id() as usize)
            .map(|a| a.remove(entity))
            .unwrap_or(false)
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.archetypes
            .get(entity.archetype_id() as usize)
            .map(|a| a.contains(entity))
            .unwrap_or(false)
    }

    pub fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        self.archetypes
            .get(entity.archetype_id() as usize)
            .map(|a| util::subset(T::type_ids().as_ref(), a.type_ids()))
            .unwrap_or(false)
    }

    pub fn get<'a, 'b, 'c, T: ComponentRefTuple<'c>>(&'a self, entity: Entity) -> Option<T>
    where
        'a: 'b,
        'b: 'c,
    {
        self.archetypes
            .get(entity.archetype_id() as usize)
            .map(|a| a.get::<'b, 'c, T>(entity))
            .flatten()
    }

    pub fn get_mut<'a, 'b, 'c, T: ComponentBorrowTuple<'c>>(
        &'a mut self,
        entity: Entity,
    ) -> Option<T>
    where
        'a: 'b,
        'b: 'c,
    {
        util::assert_no_alias(
            T::ValueType::type_ids().as_ref(),
            "aliasing in component tuple",
        );
        self.archetypes
            .get_mut(entity.archetype_id() as usize)
            .map(|a| a.get_mut::<'b, 'c, T>(entity))
            .flatten()
    }

    pub fn unsafe_cell(&self) -> UnsafeRegistryCell<'_> {
        UnsafeRegistryCell(self)
    }
}

#[derive(Clone)]
pub struct UnsafeRegistryCell<'a>(pub &'a Registry);

unsafe impl<'a> Send for UnsafeRegistryCell<'a> {}
unsafe impl<'a> Sync for UnsafeRegistryCell<'a> {}

impl<'a> UnsafeRegistryCell<'a> {
    pub fn messages<'b>(&self) -> &'b Messages
    where
        'a: 'b,
    {
        self.0.messages()
    }

    pub fn archetypes(&self) -> &'a [Archetype] {
        self.0.archetypes()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.0.contains(entity)
    }

    pub fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        self.0.has::<T>(entity)
    }

    pub unsafe fn get<'b, T: ComponentRefTuple<'b>>(&self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        self.0.get(entity)
    }

    pub unsafe fn get_mut<'b, T: ComponentBorrowTuple<'b>>(&self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        util::assert_no_alias(
            T::ValueType::type_ids().as_ref(),
            "aliasing in component tuple",
        );
        self.0
            .archetypes
            .get(entity.archetype_id() as usize)
            .map(|a| UnsafeArchetypeCell(a).get_mut::<T>(entity))
            .flatten()
    }
}

impl Debug for Registry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug_list = f.debug_list();
        for a in self.archetypes.iter() {
            for i in 0..a.entities().len() {
                debug_list.entry(&DebugArchetypeEntry {
                    archetype: a,
                    index: i,
                });
            }
        }
        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::{component::Component, registry::Registry};

    #[derive(Debug)]
    pub struct ComponentA(i64);
    #[derive(Debug)]
    pub struct ComponentB(i64);
    #[derive(Debug)]
    pub struct ComponentC(i64);

    impl Component for ComponentA {}
    impl Component for ComponentB {}
    impl Component for ComponentC {}

    #[test]
    fn test() {
        let mut reg = Registry::new();
        let e1 = reg.push((ComponentA(1), ComponentB(1), ComponentC(1)));
        let e2 = reg.push((ComponentA(2), ComponentB(2), ComponentC(2)));
        let e3 = reg.push((ComponentA(3), ComponentB(3), ComponentC(3)));
        let e4 = reg.push((ComponentA(4), ComponentB(4), ComponentC(4)));
        let e5 = reg.push((ComponentB(5), ComponentC(5)));
        let e6 = reg.push((ComponentA(6), ComponentC(6)));
        let e7 = reg.push((ComponentA(7), ComponentB(7)));
        // println!("{:#?}", reg);
        reg.remove(e2);
        reg.remove(e3);
        reg.remove(e7);
        println!("{:#?}", reg);

        for i in &[e1, e2, e3, e4] {
            // reg.get_mut::<(&mut ComponentA,)>(*i);
            reg.get_mut::<(&mut ComponentA,)>(*i);
        }

        let a = |x: i32| {
            println!("{}", x);
        };
    }
}
