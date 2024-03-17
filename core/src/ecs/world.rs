use std::fmt::{Debug, Formatter};
use itertools::izip;

use crate::ecs::{
    component::ComponentSet,
    entity::{DebugEntity, Entity},
    storage::archetype::ArchetypeStorage,
    system::{System, SystemAccessor, SystemMut, SystemPermissions},
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    ArchetypeId, Index,
};

pub struct World {
    pub archetypes: Vec<ArchetypeStorage>,
    pub archetype_lookup: hashbrown::HashMap<ComponentSet, ArchetypeId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            archetype_lookup: hashbrown::HashMap::new(),
        }
    }

    pub fn get_components<'a, T: ComponentRefTuple<'a>>(&'a self, entity: Entity) -> Option<T> {
        if let Some(archetype) = self.archetypes.get(entity.archetype_id() as usize) {
            if T::ValueType::COMPONENT_SET.subset(&archetype.component_set) {
                unsafe {
                    return archetype.get_components(entity);
                }
            }
        }
        None
    }

    pub fn get_components_mut<'a, T: ComponentBorrowTuple<'a>>(
        &'a mut self,
        entity: Entity,
    ) -> Option<T> {
        if T::ValueType::COMPONENT_SET.count_ones() != T::ValueType::COMPONENT_COUNT {
            panic!("type aliasing in component tuple");
        } else if let Some(archetype) = self.archetypes.get(entity.archetype_id() as usize) {
            if T::ValueType::COMPONENT_SET.subset(&archetype.component_set) {
                unsafe {
                    return archetype.get_components_mut(entity);
                }
            }
        }
        None
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        if T::COMPONENT_SET.count_ones() != T::COMPONENT_COUNT {
            panic!("type aliasing in component tuple");
        }
        let archetype_id = if let Some(archetype_id) = self.archetype_lookup.get(&T::COMPONENT_SET)
        {
            *archetype_id
        } else {
            let archetype_id = self.archetypes.len() as ArchetypeId;
            let archetype = ArchetypeStorage::new::<T>(archetype_id);
            self.archetypes.push(archetype);
            archetype_id
        };
        let archetype = unsafe { self.archetypes.get_unchecked_mut(archetype_id as usize) };
        unsafe { archetype.push(values) }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get_mut(entity.archetype_id() as usize) {
            archetype.remove(entity)
        } else {
            false
        }
    }

    // pub fn run_par_chained<'a, S: SystemChain<'a> + Sync>(&'a mut self, systems: &S) {
    //     if S::GLOBAL_WRITE_COMPONENT_SET.count_ones() > 0 {
    //         panic!("global write components in non-mut system");
    //     } else if !S::GLOBAL_READ_COMPONENT_SET.disjoint(&S::LOCAL_WRITE_COMPONENT_SET) {
    //         panic!("global read components alias with local write components");
    //     } else {
    //         rayon::scope(|s| {
    //             for archetype in self.archetypes.iter() {
    //                 if <S::Local as ComponentBorrowTuple>::ValueType::COMPONENT_SET.subset(&archetype.component_set) {
    //
    //                 }
    //             }
    //         });
    //     }
    // }

    pub fn run_par<'a, 'b,  S: System<'b> + Sync>(&'a mut self, system: &S) where 'a: 'b {
        if S::Global::WRITE_COMPONENT_SET.count_ones() > 0 {
            panic!("global write components in non-mut system");
        } else if !S::Global::READ_COMPONENT_SET.disjoint(&S::Local::WRITE_COMPONENT_SET) {
            panic!("global read components alias with local write components");
        } else {
            let system_permissions = SystemPermissions {
                read_local: S::Local::READ_COMPONENT_SET,
                write_local: S::Local::WRITE_COMPONENT_SET,
                read_global: S::Global::READ_COMPONENT_SET,
                write_global: ComponentSet::zeros(),
            };
            rayon::scope(|s| {
                for archetype in self.archetypes.iter() {
                    if <S::Local as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                        .subset(&archetype.component_set)
                    {
                        for chunk in archetype.chunks(1024) {
                            let mut system_accessor =
                                SystemAccessor::new(self, &system_permissions);
                            s.spawn(move |_| {
                                for (index, entity) in chunk.entities().iter().enumerate() {
                                    let index = index as Index;
                                    unsafe {
                                        system_accessor.set_current(*entity);
                                        system.run(
                                            *entity,
                                            S::Local::get_components(archetype, index),
                                            &mut system_accessor,
                                        );
                                    }
                                }
                            });
                        }
                    }
                }
            });
        }
    }

    pub fn run_mut<'a, S: SystemMut<'a>>(&'a mut self, system: &mut S) {
        let system_permissions = SystemPermissions {
            read_local: S::Local::READ_COMPONENT_SET,
            write_local: S::Local::WRITE_COMPONENT_SET,
            read_global: S::Global::READ_COMPONENT_SET,
            write_global: S::Global::WRITE_COMPONENT_SET,
        };
        let mut system_accessor = SystemAccessor::new(self, &system_permissions);
        for archetype in self.archetypes.iter() {
            if <S::Local as ComponentBorrowTuple>::ValueType::COMPONENT_SET
                .subset(&archetype.component_set)
            {
                for (index, entity) in archetype.entities.iter().enumerate() {
                    let index = index as Index;
                    unsafe {
                        system_accessor.set_current(*entity);
                        system.run(
                            *entity,
                            S::Local::get_components(archetype, index),
                            &mut system_accessor,
                        );
                    }
                }
            }
        }
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list_fmt = f.debug_list();
        for archetype in self.archetypes.iter() {
            for entity in archetype.entities.iter() {
                list_fmt.entry(&DebugEntity { archetype, entity });
            }
        }
        list_fmt.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{
        component::Component,
        entity::Entity,
        system::{System, SystemAccessor, SystemMut},
        world::World,
        ComponentId,
    };

    #[derive(Debug)]
    pub struct Position(i32, i32, i32);
    #[derive(Debug)]
    pub struct Velocity(i32, i32, i32);

    #[derive(Debug)]
    pub struct Name(&'static str);

    unsafe impl Component for Position {
        const ID: ComponentId = 0;
    }

    unsafe impl Component for Velocity {
        const ID: ComponentId = 1;
    }

    unsafe impl Component for Name {
        const ID: ComponentId = 2;
    }

    pub struct PhysicsSystem;

    impl<'a> SystemMut<'a> for PhysicsSystem {
        type Global = (&'a mut Velocity,);
        type Local = (&'a mut Position, &'a Velocity);

        fn run(
            &mut self,
            entity: Entity,
            components: Self::Local,
            system_accessor: &mut SystemAccessor,
        ) {
            let (p, v) = components;
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
            let other = Entity::new(2, 0, 0);
            if other != entity {
                let name = system_accessor.get_components_mut::<(&mut Velocity,)>(other);
                println!("{:?}", name);
            }
        }
    }

    pub struct PhysicsSystem2;

    impl<'a> System<'a> for PhysicsSystem2 {
        type Global = ();
        type Local = (&'a mut Position, &'a Velocity);

        fn run(
            &self,
            entity: Entity,
            components: Self::Local,
            system_accessor: &mut SystemAccessor,
        ) {
            let (p, v) = components;
            p.0 += v.0;
            p.1 += v.1;
            p.2 += v.2;
        }
    }

    #[test]
    fn test() {
        let mut world = World::new();
        let e1 = world.push((Position(1, 1, 1), Velocity(1, 1, 1)));
        let e2 = world.push((Position(2, 2, 2), Name("sheep")));
        let e3 = world.push((Position(3, 3, 3), Velocity(3, 3, 3), Name("zombie")));
        let e4 = world.push((Position(4, 4, 4),));
        let e5 = world.push((Position(5, 5, 5),));
        let e6 = world.push((Position(6, 6, 6),));

        world.remove(e5);
        let e7 = world.push((Position(7, 7, 7),));
        // println!("{:#?}", world);
        // let mut physics_system = PhysicsSystem;
        // world.run_mut(&mut physics_system);
        let physics_system = PhysicsSystem2;
        world.run_par(&physics_system);

        println!("{:#?}", world);
    }
}
