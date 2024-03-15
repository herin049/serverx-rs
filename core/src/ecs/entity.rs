use crate::ecs::{
    component::{Component, ComponentSet},
    storage::ComponentVec,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    world::{World, WorldAccessor},
    Generation, Index,
};

pub struct Entity(pub Index, pub Generation);

pub struct EntityRef<'a> {
    pub accessor: WorldAccessor<'a>,
    pub index: Index,
}

impl<'a> EntityRef<'a> {
    pub fn has_components<T: ComponentTuple>(&self) -> bool {
        WorldAccessor::has_components::<T>(&self.accessor, self.index)
    }

    pub fn get_components<T: ComponentRefTuple>(&self) -> T {
        WorldAccessor::get_components(&self.accessor, self.index).unwrap()
    }

    pub fn has_component<T: Component>(&self) -> bool {
        WorldAccessor::has_component::<T>(&self.accessor, self.index)
    }

    pub fn get_component<T: Component>(&self) -> &T {
        WorldAccessor::get_component::<T>(&self.accessor, self.index).unwrap()
    }
}

pub struct EntityMut<'a> {
    pub accessor: WorldAccessor<'a>,
    pub index: Index,
}

impl<'a> EntityMut<'a> {
    pub fn has_components<T: ComponentTuple>(&self) -> bool {
        WorldAccessor::has_components::<T>(&self.accessor, self.index)
    }

    pub fn get_components<T: ComponentRefTuple>(&self) -> T {
        WorldAccessor::get_components(&self.accessor, self.index).unwrap()
    }

    pub fn get_components_mut<T: ComponentBorrowTuple>(&mut self) -> T {
        WorldAccessor::get_components_mut(&mut self.accessor, self.index).unwrap()
    }

    pub fn has_component<T: Component>(&self) -> bool {
        WorldAccessor::has_component::<T>(&self.accessor, self.index)
    }

    pub fn get_component<T: Component>(&self) -> &T {
        WorldAccessor::get_component::<T>(&self.accessor, self.index).unwrap()
    }

    pub fn get_component_mut<T: Component>(&mut self) -> &mut T {
        WorldAccessor::get_component_mut::<T>(&mut self.accessor, self.index).unwrap()
    }
}
