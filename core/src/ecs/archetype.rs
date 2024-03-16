use std::fmt::{Debug, Formatter};

use slab::Slab;

use crate::ecs::{
    component::{Component, ComponentSet},
    entity::Entity,
    storage::archetype::ArchetypeStorage,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    ArchetypeId, Generation, Index,
};

pub struct Archetype {
    pub archetype_id: ArchetypeId,
    pub components: ComponentSet,
    pub storage: ArchetypeStorage,
    pub entities: Vec<Entity>
}

impl Archetype {
    pub fn new<T: ComponentTuple>(archetype_id: ArchetypeId) -> Self {
        Self {
            archetype_id,
            components: T::COMPONENT_SET,
            storage: ArchetypeStorage::new::<T>(),
            freelist: Slab::new(),
            generations: Vec::new(),
        }
    }

    pub unsafe fn get_components<'a, T: ComponentRefTuple<'a>>(
        &'a self,
        entity: Entity,
    ) -> Option<T> {
        if self.freelist.contains(entity.index() as usize) {
            unsafe {
                if *self.generations.get_unchecked(entity.index() as usize) != entity.generation() {
                    return None;
                }
                Some(self.storage.get_components_unchecked(entity.index()))
            }
        } else {
            None
        }
    }

    pub unsafe fn get_components_mut<'a, T: ComponentBorrowTuple<'a>>(
        &'a self,
        entity: Entity,
    ) -> Option<T> {
        if self.freelist.contains(entity.index() as usize) {
            unsafe {
                if *self.generations.get_unchecked(entity.index() as usize) != entity.generation() {
                    return None;
                }
                Some(self.storage.get_components_mut_unchecked(entity.index()))
            }
        } else {
            None
        }
    }

    pub unsafe fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let index = self.freelist.insert(()) as Index;
        if (index as usize) >= self.generations.len() {
            self.generations.resize((index as usize) + 1, 0);
        }
        unsafe {
            let generation = *self.generations.get_unchecked(index as usize);
            self.storage.insert_unchecked(index, values);
            Entity::new(index, generation, self.archetype_id)
        }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if self.freelist.try_remove(entity.index() as usize).is_some() {
            let generation = unsafe { self.generations.get_unchecked_mut(entity.index() as usize) };
            if entity.generation() != *generation {
                return false;
            };
            *generation = generation.wrapping_add(1);
            self.components.iter().for_each(|c| unsafe {
                self.storage
                    .components
                    .get_unchecked_mut(c as usize)
                    .assume_init_mut()
                    .drop_at_unchecked(entity.index());
            });
            true
        } else {
            false
        }
    }
}

impl Debug for Archetype {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, _) in self.freelist.iter() {
            let entity = Entity::new(index as Index, self.generations[index], self.archetype_id);
            write!(f, "[ {} ", entity)?;
            for component_id in self.components.iter() {
                unsafe {
                    self.storage
                        .components
                        .get_unchecked(component_id as usize)
                        .assume_init_ref()
                        .fmt_at_unchecked(f, index as Index)?;
                }
                write!(f, " ")?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

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

    use crate::ecs::{archetype::Archetype, component::Component, ArchetypeId, ComponentId};

    #[test]
    fn test() {
        unsafe {
            let mut a = Archetype::new::<(Position, Velocity)>(0 as ArchetypeId);
            println!("{:?}", a);
            let e1 = a.push((Position(1, 2, 3), Velocity(4, 5, 6)));
            let e2 = a.push((Position(2, 4, 6), Velocity(1, 2, 3)));
            let e3 = a.push((Position(1, 22, 3), Velocity(4, 10, 6)));
            println!("{:?}", a);
            println!("{} {} {}", e1, e2, e3);
            a.remove(e2);
            println!("{:?}", a);
            unsafe {
                if let Some((p, v)) = a.get_components_mut::<(&mut Position, &Velocity)>(e1) {
                    println!("{:?} {:?}", p, v);
                    p.0 = 10000;
                }
            }
            println!("{:?}", a);
        }
    }
}
