use std::{
    any::TypeId,
    fmt::{Debug, Formatter},
    marker::PhantomData,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use slab::Slab;

use crate::{
    entity::Entity,
    event::Event,
    sort::{insertion_sort, insertion_sort_noalias},
    storage::{
        archetype::{ArchetypeIter, ArchetypeStorage, DebugArchetypeEntry},
        event::EventStorage,
    },
    system::{System, SystemAccessor, SystemMut, SystemPar},
    tuple::{
        ptr::PtrTuple, type_tuple::TypeTuple, ComponentBorrowTuple, ComponentRefTuple,
        ComponentTuple,
    },
    types, ArchetypeId, ArchetypeIndex,
};
use crate::handler::{Handler, HandlerAccessor, HandlerMut, HandlerPar};
use crate::tuple::EventTuple;

pub struct Registry {
    pub archetypes: Vec<ArchetypeStorage>,
    pub archetype_lookup: Vec<(Box<[TypeId]>, ArchetypeId)>,
    pub events: EventStorage,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            archetype_lookup: Vec::new(),
            events: EventStorage::new(),
        }
    }

    pub fn find(&self, entity: Entity) -> Option<(&ArchetypeStorage, ArchetypeIndex)> {
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

    pub fn has<T: ComponentTuple>(&self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get(entity.archetype_id() as usize) {
            types::subset(T::type_ids().as_ref(), archetype.type_ids.as_ref())
        } else {
            false
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.find(entity).is_some()
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
        if let Some((archetype, index)) = self.find(entity) {
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
        let mut type_ids = <T as ComponentBorrowTuple<'b>>::ValueType::type_ids();
        insertion_sort_noalias(type_ids.as_mut());
        if let Some((archetype, index)) = self.find(entity) {
            if let Ok(ptr) =
                archetype.try_as_mut_ptr::<<T as ComponentBorrowTuple<'b>>::ValueType>()
            {
                unsafe {
                    return Some(<T as ComponentBorrowTuple<'b>>::deref(
                        ptr.offset(index as isize),
                    ));
                }
            }
        }
        None
    }

    pub fn run_par<'a, 'b, S: SystemPar<'b> + Sync + 'b>(&'a mut self, system: &S)
    where
        'a: 'b,
        S: Send,
        <<S as SystemPar<'b>>::Local as ComponentBorrowTuple<'b>>::ValueType: Sync,
        S::Global: ComponentRefTuple<'b>,
    {
        if !types::disjoint(
            <S::Global as ComponentBorrowTuple<'b>>::ReadType::type_ids().as_ref(),
            <S::Local as ComponentBorrowTuple<'b>>::WriteType::type_ids().as_ref(),
        ) {
            panic!("global read set aliases with local write set");
        }
        {
            S::Send::register(&mut self.events);
        }
        {
            let r = rayon::scope(|s| {
                for archetype in self.archetypes.iter() {
                    if types::subset(
                        <S::Local as ComponentBorrowTuple<'b>>::ValueType::type_ids().as_ref(),
                        archetype.type_ids.as_ref(),
                    ) {
                        unsafe {
                            for chunk in archetype.chunks_mut::<S::Local>(4096) {
                                s.spawn(|_| {
                                    let mut accessor =
                                        SystemAccessor::<'_, 'b, S::Local, S::Global>::new(self, true);
                                    let mut ctx = system.par_ctx();
                                    unsafe {
                                        for (entity, values) in chunk {
                                            accessor.update_entity(entity);
                                            system.run(entity, values, &mut accessor, &mut ctx);
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            });
        }
        S::Send::sync(&mut self.events);
    }

    pub fn run<'a, S: System<'a> + 'a>(&'a mut self, system: &S) {
        S::Send::register(&mut self.events);
        let mut accessor = SystemAccessor::<'_, 'a, S::Local, S::Global>::new(self, false);
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

    pub fn run_mut<'a, S: SystemMut<'a> + 'a>(&'a mut self, system: &mut S) {
        S::Send::register(&mut self.events);
        let mut accessor = SystemAccessor::<'_, 'a, S::Local, S::Global>::new(self, false);
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

    pub fn handle<'a, 'b, H: Handler<'b> + 'b>(&'a mut self, handler: &H) {
        if H::Send::type_ids().as_ref().iter().find(|x| TypeId::of::<H::Event>().eq(*x)).is_some() {
            panic!("alias in event handler types");
        }
        let mut accessor = HandlerAccessor::<'_, 'b, H::Global, H::Send>::new(self, false);
        unsafe {
            for e in self.events.events::<H::Event>() {
                handler.run(e, &mut accessor);
            }
        }
    }

    pub fn handle_mut<'a, 'b, H: HandlerMut<'b> + 'b>(&'a mut self, handler: &mut H) {
        if H::Send::type_ids().as_ref().iter().find(|x| TypeId::of::<H::Event>().eq(*x)).is_some() {
            panic!("alias in event handler types");
        }
        let mut accessor = HandlerAccessor::<'_, 'b, H::Global, H::Send>::new(self, false);
        unsafe {
            for e in self.events.events::<H::Event>() {
                handler.run(e, &mut accessor);
            }
        }
    }

    pub fn handle_par<'a, 'b, H: HandlerPar<'b> + Sync + 'b>(&'a mut self, handler: &'b H) where <H as HandlerPar<'b>>::Event: Sync {
        if H::Send::type_ids().as_ref().iter().find(|x| TypeId::of::<H::Event>().eq(*x)).is_some() {
            panic!("alias in event handler types");
        }
        let chunks = unsafe { self.events.events::<H::Event>().chunks(1024) };
        rayon::scope(|s| {
            unsafe {
                for c in chunks {
                    let handler_ref = &*handler;
                    let self_ref = &*self;
                    s.spawn(move |_| {
                        let mut ctx = handler_ref.par_ctx();
                        let mut accessor = HandlerAccessor::<'_, 'b, H::Global, H::Send>::new(self_ref, false);
                        for e in c {
                            handler_ref.run(e, &mut accessor, &mut ctx);
                        }
                    });
                }
            }
        });
        H::Send::sync(&mut self.events);
    }

    pub fn register_event<T: Event>(&mut self) {
        self.events.register::<T>();
    }

    pub fn send<T: Event>(&mut self, e: T) {
        unsafe {
            self.events.send(e);
        }
    }
}

unsafe impl Sync for Registry {}

impl Debug for Registry {
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

#[cfg(test)]
mod tests {
    use crate::{
        entity::Entity,
        registry::Registry,
        system::{System, SystemAccessor, SystemMut, SystemPar},
    };

    pub struct TestSystem;

    impl<'a> System<'a> for TestSystem {
        type Global = ();
        type Local = (&'a i32, &'a mut f32);
        type Send = ();

        fn run(
            &self,
            entity: Entity,
            components: Self::Local,
            accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
        ) {
            println!("{:?} {:?} {:?}", entity, components.0, components.1);
            *components.1 *= 2.0;
        }

    }

    impl<'a> SystemMut<'a> for TestSystem {
        type Global = ();
        type Local = (&'a i32, &'a mut f32);
        type Send = ();

        fn run(
            &mut self,
            entity: Entity,
            components: Self::Local,
            accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
        ) {
            println!("{:?} {:?} {:?}", entity, components.0, components.1);
            *components.1 *= 2.0;
        }
    }

    pub struct TestParSystem {
        pub value: i32,
    }
    impl<'a> SystemPar<'a> for TestParSystem {
        type Ctx = i32;
        type Global = ();
        type Local = (&'a i32, &'a mut f32);
        type Send = ();

        fn par_ctx(&self) -> Self::Ctx {
            self.value
        }

        fn run(
            &self,
            entity: Entity,
            components: Self::Local,
            accessor: &mut SystemAccessor<'_, 'a, Self::Local, Self::Global>,
            ctx: &mut Self::Ctx,
        ) {
            println!("{:?} {:?} {:?}", entity, components.0, components.1);
            *components.1 *= 2.0;
        }
    }

    #[test]
    fn test_system() {
        let mut world = Registry::new();
        let e1 = world.push((13i32, 16i64, 14.5f32, "hello world1"));
        let e2 = world.push((13i32, 16i64, 14.5f32, "hello world2"));
        let e3 = world.push((13i32, 16i64, 14.5f32, "hello world3"));
        let e4 = world.push((13i32, 16i64, 14.5f32, "hello world4"));
        let e5 = world.push((13i32, 16i64, 14.5f32, "hello world5"));
        let e6 = world.push((13i32, 16i64, 14.5f32, "hello world6"));
        let mut system = TestSystem;
        world.run(&system);
        world.run_par(&TestParSystem { value: 3 });
        world.run_mut(&mut system);
        println!("{:#?}", world);
    }

    #[test]
    fn test() {
        let mut world = Registry::new();
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
