use std::{
    any::TypeId,
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use slab::Slab;

use crate::{
    entity::Entity,
    sort::{insertion_sort, insertion_sort_noalias},
    storage::archetype::{ArchetypeStorage, DebugArchetypeEntry},
    system::{System, SystemAccessor, SystemMut},
    tuple::{
        ptr::PtrTuple, type_tuple::TypeTuple, ComponentBorrowTuple, ComponentRefTuple,
        ComponentTuple,
    },
    types, ArchetypeId, ArchetypeIndex,
};

pub struct World {
    pub archetypes: Vec<ArchetypeStorage>,
    pub archetype_lookup: Vec<(Box<[TypeId]>, ArchetypeId)>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            archetype_lookup: Vec::new(),
        }
    }

    pub fn lookup_entity(&self, entity: Entity) -> Option<(&ArchetypeStorage, ArchetypeIndex)> {
        if let Some(archetype) = self.archetypes.get(entity.archetype_id() as usize) {
            if let Some(index) = archetype
                .entity_lookup
                .get(entity.archetype_index() as usize)
            {
                let e = unsafe { archetype.entities.get_unchecked(*index as usize) };
                if e.generation() == entity.generation() {
                    return Some((archetype, *index));
                }
            }
        }
        None
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get_mut(entity.archetype_id() as usize) {
            archetype.remove_entity(entity)
        } else {
            false
        }
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let mut tys = T::type_ids();
        insertion_sort(tys.as_mut());
        let m = self
            .archetype_lookup
            .binary_search_by_key(&tys.as_ref(), |x| x.0.as_ref());
        match m {
            Ok(i) => unsafe {
                let archetype_id = self.archetype_lookup.get_unchecked(i).1;
                self.archetypes
                    .get_unchecked_mut(archetype_id as usize)
                    .push_entity(values)
            },
            Err(i) => unsafe {
                let archetype_id = self.archetypes.len() as ArchetypeId;
                self.archetypes
                    .push(ArchetypeStorage::new::<T>(archetype_id));
                self.archetype_lookup.insert(i, (tys.into(), archetype_id));
                self.archetypes
                    .get_unchecked_mut(archetype_id as usize)
                    .push_entity(values)
            },
        }
    }

    pub fn has_components<T: ComponentTuple>(&self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get(entity.archetype_id() as usize) {
            types::subset(T::type_ids().as_ref(), archetype.type_ids.as_ref())
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.lookup_entity(entity).is_some()
    }

    pub fn get<'a, 'b, T: ComponentRefTuple<'b> + ComponentBorrowTuple<'b>>(
        &'a self,
        entity: Entity,
    ) -> Option<T>
    where
        'a: 'b,
    {
        let mut type_ids = <T as ComponentRefTuple<'b>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if let Some((archetype, index)) = self.lookup_entity(entity) {
            if let Ok(ptr) = archetype.try_as_mut_ptr::<<T as ComponentRefTuple<'b>>::ValueType>() {
                unsafe {
                    return Some(<T as ComponentRefTuple<'b>>::deref(
                        ptr.offset(index as isize),
                    ));
                }
            }
        }
        None
    }

    pub fn get_mut<'a, 'b, T: ComponentBorrowTuple<'b>>(&mut self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        let mut type_ids = <T as ComponentRefTuple<'b>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if let Some((archetype, index)) = self.lookup_entity(entity) {
            if let Ok(ptr) = archetype.try_as_mut_ptr::<<T as ComponentRefTuple<'b>>::ValueType>() {
                unsafe {
                    return Some(<T as ComponentBorrowTuple<'b>>::deref(
                        ptr.offset(index as isize),
                    ));
                }
            }
        }
        None
    }

    pub fn run_par<'a, S: System<'a> + Sync>(&'a mut self, system: &S)
    where
        S: Send,
        <<S as System<'a>>::Local as ComponentBorrowTuple<'a>>::ValueType: Sync,
        S::Global: ComponentRefTuple<'a>,
    {
        if !types::disjoint(
            <S::Global as ComponentBorrowTuple<'a>>::ReadType::type_ids().as_ref(),
            <S::Local as ComponentBorrowTuple<'a>>::WriteType::type_ids().as_ref(),
        ) {
            panic!("global read set aliases with local write set");
        }
        rayon::scope(|s| {
            for archetype in self.archetypes.iter() {
                if types::subset(
                    <S::Local as ComponentBorrowTuple<'a>>::ValueType::type_ids().as_ref(),
                    archetype.type_ids.as_ref(),
                ) {
                    unsafe {
                        for chunk in archetype.chunks_mut::<S::Local>(4096) {
                            s.spawn(|_| {
                                let mut accessor =
                                    SystemAccessor::<'a, S::Local, S::Global>::new(self);
                                unsafe {
                                    for (entity, values) in chunk {
                                        accessor.update_entity(entity);
                                        system.run(entity, values, &mut accessor);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });
    }

    pub fn run<'a, S: System<'a>>(&'a mut self, system: &S) {
        let mut accessor = SystemAccessor::<'a, S::Local, S::Global>::new(self);
        for archetype in self.archetypes.iter() {
            if types::subset(
                <S::Local as ComponentBorrowTuple<'a>>::ValueType::type_ids().as_ref(),
                archetype.type_ids.as_ref(),
            ) {
                unsafe {
                    for (entity, values) in archetype.iter_mut::<S::Local>() {
                        accessor.update_entity(entity);
                        system.run(entity, values, &mut accessor);
                    }
                }
            }
        }
    }

    pub fn run_mut<'a, S: SystemMut<'a>>(&'a mut self, system: &mut S) {
        let mut accessor = SystemAccessor::<'a, S::Local, S::Global>::new(self);
        for archetype in self.archetypes.iter() {
            if types::subset(
                <S::Local as ComponentBorrowTuple<'a>>::ValueType::type_ids().as_ref(),
                archetype.type_ids.as_ref(),
            ) {
                unsafe {
                    for (entity, values) in archetype.iter_mut::<S::Local>() {
                        accessor.update_entity(entity);
                        system.run(entity, values, &mut accessor);
                    }
                }
            }
        }
    }
}

unsafe impl Sync for World {}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        for arch in self.archetypes.iter() {
            (0..arch.entities.len()).for_each(|i| {
                debug_list.entry(&DebugArchetypeEntry {
                    archetype_storage: arch,
                    index: i,
                });
            });
        }
        debug_list.finish()
    }
}

pub struct WorldAccess<'a, T: ComponentBorrowTuple<'a>> {
    pub world: &'a World,
    phantom: PhantomData<T>,
}

impl<'a, T: ComponentBorrowTuple<'a>> WorldAccess<'a, T> {
    pub unsafe fn new(world: &'a World) -> Self {
        Self {
            world,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entity::Entity,
        system::{System, SystemAccessor, SystemMut},
        world::World,
    };

    pub struct TestSystem;

    impl<'a> System<'a> for TestSystem {
        type Global = ();
        type Local = (&'a i32, &'a mut f32);

        fn run(
            &self,
            entity: Entity,
            components: Self::Local,
            accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
        ) {
            println!("{:?} {:?} {:?}", entity, components.0, components.1);
            *components.1 *= 2.0;
        }
    }

    impl<'a> SystemMut<'a> for TestSystem {
        type Global = ();
        type Local = (&'a i32, &'a mut f32);

        fn run(
            &mut self,
            entity: Entity,
            components: Self::Local,
            accessor: &mut SystemAccessor<'a, Self::Local, Self::Global>,
        ) {
            println!("{:?} {:?} {:?}", entity, components.0, components.1);
            *components.1 *= 2.0;
        }
    }

    #[test]
    fn test_system() {
        let mut world = World::new();
        let e1 = world.push((13i32, 16i64, 14.5f32, "hello world1"));
        let e2 = world.push((13i32, 16i64, 14.5f32, "hello world2"));
        let e3 = world.push((13i32, 16i64, 14.5f32, "hello world3"));
        let e4 = world.push((13i32, 16i64, 14.5f32, "hello world4"));
        let e5 = world.push((13i32, 16i64, 14.5f32, "hello world5"));
        let e6 = world.push((13i32, 16i64, 14.5f32, "hello world6"));
        let mut system = TestSystem;
        // world.run(&system);
        // world.run_par(&system);
        world.run_mut(&mut system);
        println!("{:#?}", world);
    }

    #[test]
    fn test() {
        let mut world = World::new();
        let e1 = world.push((13i32, 16i64, 14.5f32, "hello world1"));
        let e2 = world.push((13i32, 16i64, 14.5f32, "hello world2"));
        let e3 = world.push((13i32, 16i64, 14.5f32, "hello world3"));
        let e4 = world.push((13i32, 16i64, 14.5f32, "hello world4"));
        let e5 = world.push((13i32, 16i64, 14.5f32, "hello world5"));
        let e6 = world.push((13i32, 16i64, 14.5f32, "hello world6"));
        println!("{:#?}", world);
        world.remove(e2);
        world.remove(e4);
        println!("{:#?}", world);
        world.remove(e1);
        world.remove(e3);
        world.remove(e5);
        println!("{:#?}", world);
    }
}
