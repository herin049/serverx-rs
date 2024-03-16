use std::mem::MaybeUninit;

use crate::ecs::{
    entity::Entity,
    storage::component::ComponentStorage,
    tuple::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
    Index,
};

pub struct ArchetypeStorage {
    pub components: Vec<MaybeUninit<Box<dyn ComponentStorage>>>,
}

impl ArchetypeStorage {
    pub fn new<T: ComponentTuple>() -> Self {
        let mut components = Vec::new();
        for component_id in T::COMPONENT_SET.iter() {
            if (component_id as usize) >= components.len() {
                let diff = (component_id as usize + 1) - components.len();
                components.reserve(diff);
                unsafe {
                    components.set_len(component_id as usize + 1);
                }
            }
        }
        let mut storage = ArchetypeStorage { components };
        unsafe {
            T::init_storage_unchecked(&mut storage);
        }
        storage
    }

    pub unsafe fn get_components_unchecked<'a, T: ComponentRefTuple<'a>>(
        &'a self,
        index: Index,
    ) -> T {
        T::get_unchecked(self, index)
    }

    pub unsafe fn get_components_mut_unchecked<'a, T: ComponentBorrowTuple<'a>>(
        &'a self,
        index: Index,
    ) -> T {
        T::get_unchecked(self, index)
    }

    pub unsafe fn insert_unchecked<T: ComponentTuple>(&mut self, index: Index, values: T) {
        T::insert_unchecked(values, self, index)
    }
}
