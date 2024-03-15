use std::{
    fmt::{Debug, Formatter},
    mem,
    ops::Range,
};

use itertools::Itertools;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use slab::Slab;

use crate::ecs::{
    component::{Component, ComponentId, ComponentSet},
    entity::{Entity, EntityMut, EntityRef},
    storage::{sync_unsafe_cell::SyncUnsafeCell, ComponentStorage, ComponentVec},
    system::System,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    Generation, Index,
};

pub struct WorldBuilder {
    pub components: Vec<(ComponentId, Box<dyn ComponentStorage>)>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn register<T: Component>(&mut self) -> &mut Self {
        self.components
            .push((T::ID, Box::new(ComponentVec::<T>::new())));
        self
    }

    pub fn build(mut self) -> World {
        self.components.sort_by_key(|(i, _)| *i);
        let world_components = self
            .components
            .into_iter()
            .enumerate()
            .map(|(i, (j, c))| {
                if i != (j as usize) {
                    panic!("component with id {} was not registered", i);
                }
                SyncUnsafeCell::new(c)
            })
            .collect_vec();
        World {
            entities: SyncUnsafeCell::new(Slab::new()),
            generation: 0,
            component_sets: SyncUnsafeCell::new(Vec::new()),
            components: SyncUnsafeCell::new(world_components),
        }
    }
}

pub struct World {
    pub entities: SyncUnsafeCell<Slab<Generation>>,
    pub generation: Generation,
    pub component_sets: SyncUnsafeCell<Vec<ComponentSet>>,
    pub components: SyncUnsafeCell<Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>>,
}

impl World {
    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        if T::COMPONENT_SET.count_ones() != T::COMPONENT_COUNT {
            panic!("type aliasing in component tuple");
        }
        println!("{:?}", T::COMPONENT_SET);
        let entity_entry = unsafe { (&mut *self.entities.get()).vacant_entry() };
        let index = entity_entry.key() as Index;
        let generation = self.generation;
        self.generation = self.generation.wrapping_add(1);
        entity_entry.insert(self.generation);
        unsafe {
            if index >= (&*self.component_sets.get()).len() as Index {
                (&mut *self.component_sets.get())
                    .resize((index as usize) + 1, ComponentSet::zeros());
            }
            let _ = mem::replace(
                (&mut *self.component_sets.get()).get_unchecked_mut(index as usize),
                T::COMPONENT_SET,
            );
            T::insert_unchecked(values, &*self.components.get(), index);
        }
        Entity(index, generation)
    }

    pub fn get(&self, entity: Entity) -> Option<EntityRef> {
        let present = unsafe {
            (&*self.entities.get())
                .get(entity.0 as usize)
                .map(|g| entity.1 == *g)
                .unwrap_or(false)
        };
        if present {
            let accessor = unsafe {
                WorldAccessor::new(self, WorldPermissions::Read {
                    read: ComponentSet::ones(),
                })
            };
            Some(EntityRef {
                accessor,
                index: entity.0,
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> EntityMut {
        let accessor = unsafe {
            WorldAccessor::new(self, WorldPermissions::Write {
                write: ComponentSet::ones(),
            })
        };
        EntityMut {
            accessor,
            index: entity.0,
        }
    }

    pub unsafe fn get_component_unchecked<T: Component>(&self, index: Index) -> &T {
        let components = (&*self.components.get());
        let component_vec = (&*components.get_unchecked(T::ID).get());
        component_vec
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_unchecked(index)
    }

    pub unsafe fn get_component_mut_unchecked<T: Component>(&self, index: Index) -> &mut T {
        let components = (&*self.components.get());
        let component_vec = (&*components.get_unchecked(T::ID).get());
        component_vec
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_mut_unchecked(index)
    }

    pub unsafe fn get_components_unchecked<T: ComponentRefTuple>(&self, index: Index) -> T {
        <T as ComponentBorrowTuple>::get_unchecked(&self.components, index)
    }

    pub unsafe fn get_components_mut_unchecked<T: ComponentBorrowTuple>(&self, index: Index) -> T {
        T::get_unchecked(&self.components, index)
    }

    pub fn run<S: for<'a> System<'a>>(&mut self, system: &mut S) {
        if S::Local::WRITE_COMPONENT_SET
            .disjoint(&<S::Global as ComponentBorrowTuple>::ValueType::COMPONENT_SET)
        {
            let mut accessor = WorldAccessor {
                world: &self,
                permissions: WorldPermissions::ReadWrite {
                    read: S::Global::READ_COMPONENT_SET.clone(),
                    write: S::Global::WRITE_COMPONENT_SET.clone(),
                },
            };
            unsafe {
                for (index, generation) in (&*self.entities.0.get()).iter() {
                    let component_set = (&*self.component_sets.get()).get_unchecked(index);
                    if <S::Local as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                        .subset(component_set)
                    {
                        println!("{} {}", index, generation);
                        system.run_mut(
                            Entity(index as Index, *generation),
                            self.get_components_mut_unchecked::<S::Local>(index as Index),
                            &mut accessor,
                        );
                    }
                }
            }
        } else {
            panic!("system local write set and global set are not disjoint");
        }
    }

    pub fn run_par<S: for<'a> System<'a>>(&self, system: &S) {
        if S::Local::WRITE_COMPONENT_SET
            .disjoint(&<S::Global as ComponentBorrowTuple>::ValueType::COMPONENT_SET)
        {
            let mut accessor = WorldAccessor {
                world: &self,
                permissions: WorldPermissions::Read {
                    read: S::Global::READ_COMPONENT_SET.clone(),
                },
            };
            unsafe {
                (&*self.component_sets.get())
                    .as_slice()
                    .par_iter()
                    .enumerate()
                    .for_each(|(index, component_set)| {
                        if <S::Local as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                            .subset(component_set)
                        {
                            if let Some(generation) = (&*self.entities.get()).get(index) {
                                system.run(
                                    Entity(index as Index, *generation),
                                    self.get_components_mut_unchecked::<S::Local>(index as Index),
                                    &accessor,
                                );
                            }
                        }
                    });
            }
        } else {
            panic!("system local write set and global set are not disjoint");
        }
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let entities = unsafe { &*self.entities.get() };
        let component_sets = unsafe { &*self.component_sets.get() };
        let components = unsafe { &*self.components.get() };
        for i in (0usize..entities.len()) {
            write!(f, "[ ")?;
            for j in (0..components.len()) {
                if component_sets[i].contains(j as ComponentId) {
                    unsafe {
                        (&*components[j].get()).fmt_at_unchecked(f, i as Index)?;
                    }
                } else {
                    write!(f, "None")?;
                }
                write!(f, " ")?;
            }
            write!(f, "]\n");
        }
        Ok(())
    }
}

pub struct WorldPartition<'a> {
    world: &'a World,
    range: Range<Index>,
    global: ComponentSet,
    read: ComponentSet,
    write: ComponentSet,
}

pub enum WorldPermissions {
    All,
    Read {
        read: ComponentSet,
    },
    Write {
        write: ComponentSet,
    },
    ReadWrite {
        read: ComponentSet,
        write: ComponentSet,
    },
    Partition {
        range: Range<Index>,
        global: ComponentSet,
        read: ComponentSet,
        write: ComponentSet,
    },
}

pub struct WorldAccessor<'a> {
    world: &'a World,
    permissions: WorldPermissions,
}

impl<'a> WorldAccessor<'a> {
    pub unsafe fn new(world: &'a World, permissions: WorldPermissions) -> Self {
        Self { world, permissions }
    }

    pub fn has_component<T: Component>(&self, index: Index) -> bool {
        unsafe {
            (&*self.world.component_sets.get())
                .get(index as usize)
                .map(|s| s.contains(T::ID))
                .unwrap_or(false)
        }
    }

    pub fn has_components<T: ComponentTuple>(&self, index: Index) -> bool {
        unsafe {
            (&*self.world.component_sets.get())
                .get(index as usize)
                .map(|s| T::COMPONENT_SET.subset(s))
                .unwrap_or(false)
        }
    }

    pub fn get_components<T: ComponentRefTuple>(&self, index: Index) -> Option<T> {
        if self.has_components::<<T as ComponentBorrowTuple>::ValueType>(index) {
            unsafe {
                match &self.permissions {
                    WorldPermissions::All => Some(self.world.get_components_unchecked(index)),
                    WorldPermissions::Read { read } => {
                        if <T as ComponentBorrowTuple>::ValueType::COMPONENT_SET.subset(read) {
                            Some(self.world.get_components_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                    WorldPermissions::Write { write } => {
                        if <T as ComponentBorrowTuple>::ValueType::COMPONENT_SET.subset(write) {
                            Some(self.world.get_components_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                    WorldPermissions::Partition {
                        range,
                        read,
                        write,
                        global,
                    } => {
                        if range.contains(&index)
                            && <T as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                                .subset3(read, write, global)
                        {
                            Some(self.world.get_components_unchecked(index))
                        } else if <T as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                            .subset(global)
                        {
                            Some(self.world.get_components_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                    WorldPermissions::ReadWrite { read, write } => {
                        if <T as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                            .subset2(read, write)
                        {
                            Some(self.world.get_components_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn get_components_mut<T: ComponentBorrowTuple>(&mut self, index: Index) -> Option<T> {
        if T::ValueType::COMPONENT_SET.count_ones() != T::ValueType::COMPONENT_COUNT {
            panic!("type aliasing in component tuple");
        }
        if self.has_components::<T::ValueType>(index) {
            unsafe {
                match &self.permissions {
                    WorldPermissions::All => Some(self.world.get_components_mut_unchecked(index)),
                    WorldPermissions::Read { .. } => panic!("invalid write"),
                    WorldPermissions::Write { write } => {
                        if T::READ_COMPONENT_SET.subset(write)
                            && T::WRITE_COMPONENT_SET.subset(write)
                        {
                            Some(self.world.get_components_mut_unchecked(index))
                        } else {
                            panic!("invalid write");
                        }
                    }
                    WorldPermissions::Partition {
                        range,
                        write,
                        read,
                        global,
                    } => {
                        if range.contains(&index)
                            && T::READ_COMPONENT_SET.subset3(read, write, global)
                            && T::WRITE_COMPONENT_SET.subset(write)
                        {
                            Some(self.world.get_components_mut_unchecked(index))
                        } else {
                            panic!("invalid write");
                        }
                    }
                    WorldPermissions::ReadWrite { read, write } => {
                        if T::READ_COMPONENT_SET.subset2(read, write)
                            && T::WRITE_COMPONENT_SET.subset(write)
                        {
                            Some(self.world.get_components_mut_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Component>(&mut self, index: Index) -> Option<&mut T> {
        if self.has_component::<T>(index) {
            unsafe {
                match &self.permissions {
                    WorldPermissions::All => Some(self.world.get_component_mut_unchecked(index)),
                    WorldPermissions::Read { .. } => panic!("invalid write"),
                    WorldPermissions::Write { write } => {
                        if write.contains(T::ID) {
                            Some(self.world.get_component_mut_unchecked(index))
                        } else {
                            panic!("invalid write");
                        }
                    }
                    WorldPermissions::Partition { range, write, .. } => {
                        if range.contains(&index) && write.contains(T::ID) {
                            Some(self.world.get_component_mut_unchecked(index))
                        } else {
                            panic!("invalid write");
                        }
                    }
                    WorldPermissions::ReadWrite { read, write } => {
                        if write.contains(T::ID) {
                            Some(self.world.get_component_mut_unchecked(index))
                        } else {
                            panic!("invalid write");
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn get_component<T: Component>(&self, index: Index) -> Option<&T> {
        if self.has_component::<T>(index) {
            unsafe {
                match &self.permissions {
                    WorldPermissions::All => Some(self.world.get_component_unchecked(index)),
                    WorldPermissions::Read { read } => {
                        if read.contains(T::ID) {
                            Some(self.world.get_component_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                    WorldPermissions::Write { write } => {
                        if write.contains(T::ID) {
                            Some(self.world.get_component_unchecked(index))
                        } else {
                            panic!("invalid read")
                        }
                    }
                    WorldPermissions::Partition {
                        range,
                        read,
                        write,
                        global,
                    } => {
                        if global.contains(T::ID) {
                            Some(self.world.get_component_unchecked(index))
                        } else if range.contains(&index)
                            && (write.contains(T::ID) || read.contains(T::ID))
                        {
                            Some(self.world.get_component_unchecked(index))
                        } else {
                            panic!("invalid read")
                        }
                    }
                    WorldPermissions::ReadWrite { read, write } => {
                        if write.contains(T::ID) || read.contains(T::ID) {
                            Some(self.world.get_component_unchecked(index))
                        } else {
                            panic!("invalid read");
                        }
                    }
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug)]
    pub struct Position(i32, i32, i32);
    #[derive(Debug)]
    pub struct Velocity(i32, i32, i32);
    #[derive(Debug)]
    pub struct Name(String);

    unsafe impl Component for Position {
        const ID: usize = 0;
    }
    unsafe impl Component for Velocity {
        const ID: usize = 1;
    }
    unsafe impl Component for Name {
        const ID: usize = 2;
    }

    pub struct PositionSystem;

    impl Clone for PositionSystem {
        fn clone(&self) -> Self {
            PositionSystem
        }
    }

    impl<'a> System<'a> for PositionSystem {
        type Global = (&'a Name,);
        type Local = (&'a mut Position,);

        fn run_mut(
            &mut self,
            entity: Entity,
            components: Self::Local,
            world_accessor: &mut WorldAccessor,
        ) {
            components.0 .0 += 1;
            components.0 .1 += 1;
            components.0 .2 += 1;
            if let Some(name) = world_accessor.get_component::<Name>(entity.0) {
                println!("{} name={:?}", entity.0, name);
            }
        }

        fn run(&self, entity: Entity, components: Self::Local, world_accessor: &WorldAccessor) {
            components.0 .0 += 1;
            components.0 .1 += 1;
            components.0 .2 += 1;
            if let Some(name) = world_accessor.get_component::<Name>(entity.0) {
                println!("{} name={:?}", entity.0, name);
            }
        }
    }

    use crate::ecs::{
        component::Component,
        entity::Entity,
        system::System,
        world::{WorldAccessor, WorldBuilder},
    };

    #[test]
    pub fn test() {
        let mut world_builder = WorldBuilder::new();
        world_builder
            .register::<Position>()
            .register::<Velocity>()
            .register::<Name>();
        let mut world = world_builder.build();
        println!("{:?}", world);
        let entity1 = world.push((Position(1, 2, 3), Name("foobar".to_string())));
        let entity2 = world.push((Name("fizzbuzz".to_string()),));
        let entity3 = world.push((
            Name("foobuzz".to_string()),
            Velocity(1, 1, 1),
            Position(0, 0, 0),
        ));
        println!("{:?}", world);
        let mut position_system = PositionSystem;

        // world.run(&mut position_system);
        world.run_par(&position_system);
        // let entity1 = world
        // .create()
        // .with(Position(1, 2, 3))
        // .with(Name("foobar".to_string()))
        // .build();
        // let entity2 = world.create().with(Name("fizzbuzz".to_string())).build();
        // let entity3 = world.push((Name("foobar".to_string()), Velocity(1, 1, 1)));
        //
        // println!("{:?}", world);
        // let mut entitymut = world.get_mut(entity1);
        // entitymut.get_mut::<Position>().0 = 10;
        println!("{:?}", world);
    }
}
