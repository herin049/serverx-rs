use std::fmt::{Debug, Formatter};

use crate::ecs::{archetype::Archetype, component::ComponentSet, entity::Entity, tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple}, ArchetypeId, ComponentId, Index};

pub struct World {
    pub archetypes: Vec<Archetype>,
    pub archetype_lookup: hashbrown::HashMap<ComponentSet, ArchetypeId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            archetype_lookup: hashbrown::HashMap::new(),
        }
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        if T::COMPONENT_SET.count_ones() != T::COMPONENT_COUNT {
            panic!("type aliasing in component tuple");
        }
        let archetype = if let Some(archetype_id) = self.archetype_lookup.get(&T::COMPONENT_SET) {
            unsafe { self.archetypes.get_unchecked_mut(*archetype_id as usize) }
        } else {
            let archetype_id = self.archetypes.len() as ArchetypeId;
            let archetype = Archetype::new::<T>(archetype_id);
            self.archetypes.push(archetype);
            self.archetype_lookup.insert(T::COMPONENT_SET, archetype_id);
            unsafe { self.archetypes.get_unchecked_mut(archetype_id as usize) }
        };
        unsafe { archetype.push(values) }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get_mut(entity.archetype() as usize) {
            archetype.remove(entity)
        } else {
            false
        }
    }

    pub fn has_components<T: ComponentTuple>(&self, entity: Entity) -> bool {
        if let Some(archetype) = self.archetypes.get(entity.archetype() as usize) {
            T::COMPONENT_SET.subset(&archetype.components)
        } else {
            false
        }
    }

    pub fn get_components<'a, T: ComponentRefTuple<'a>>(&'a self, entity: Entity) -> Option<T> {
        if let Some(archetype) = self.archetypes.get(entity.archetype() as usize) {
            if T::ValueType::COMPONENT_SET.subset(&archetype.components) {
                unsafe { archetype.get_components(entity) }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_components_mut<'a, T: ComponentBorrowTuple<'a>>(
        &'a mut self,
        entity: Entity,
    ) -> Option<T> {
        if let Some(archetype) = self.archetypes.get_mut(entity.archetype() as usize) {
            if T::ValueType::COMPONENT_SET.subset(&archetype.components) {
                unsafe { archetype.get_components_mut(entity) }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.archetypes)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{component::Component, world::World, ComponentId};

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
        let mut w = World::new();
        let e1 = w.push((Position(1, 2, 3), Velocity(0, 0, 0)));
        let e2 = w.push((Position(2, 2, 2), Velocity(0, 0, 0)));
        let e3 = w.push((Position(3, 3, 3), Velocity(0, 0, 0)));
        let e4 = w.push((Position(4, 4, 4),));
        let e5 = w.push((Velocity(1, 1, 1),));
        println!("{:?}", w);
        let (p, v) = w
            .get_components_mut::<(&mut Position, &mut Velocity)>(e2)
            .unwrap();
        p.0 = 99999;
        v.1 = -12123;
        println!("{:?}", w);
    }
}
