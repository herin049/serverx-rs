use std::ops::Range;
use serverx_macros::component_tuple_impl;

use crate::ecs::{
    component::{Component, ComponentSet, COMPONENT_SET_LEN},
    storage::{archetype::ArchetypeStorage, component::ComponentVec},
    Index,
};

pub trait ComponentTuple {
    const COMPONENT_SET: ComponentSet;
    const COMPONENT_COUNT: usize;

    unsafe fn init_archetype_storage(storage: &mut ArchetypeStorage);
    unsafe fn push_components(self, storage: &mut ArchetypeStorage) -> Index;
}

impl ComponentTuple for () {
    const COMPONENT_COUNT: usize = 0;
    const COMPONENT_SET: ComponentSet = ComponentSet([0; COMPONENT_SET_LEN]);

    unsafe fn init_archetype_storage(storage: &mut ArchetypeStorage) {}

    unsafe fn push_components(self, storage: &mut ArchetypeStorage) -> Index {
        0 as Index
    }
}

pub trait ComponentBorrow<'a> {
    type ValueType: Component;
    const REF: bool;
    const MUT: bool;

    unsafe fn get_component(storage: &'a ArchetypeStorage, index: Index) -> Self;
}

pub trait ComponentRef<'a>: ComponentBorrow<'a> {}

impl<'a, T: Component> ComponentBorrow<'a> for &'a T {
    type ValueType = T;

    const MUT: bool = false;
    const REF: bool = true;

    unsafe fn get_component(storage: &'a ArchetypeStorage, index: Index) -> Self {
        storage
            .components
            .get_unchecked(Self::ValueType::ID as usize)
            .assume_init_ref()
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_component_unchecked(index)
    }
}

impl<'a, T: Component> ComponentBorrow<'a> for &'a mut T {
    type ValueType = T;

    const MUT: bool = true;
    const REF: bool = false;

    unsafe fn get_component(storage: &'a ArchetypeStorage, index: Index) -> Self {
        storage
            .components
            .get_unchecked(Self::ValueType::ID as usize)
            .assume_init_ref()
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_mut_component_unchecked(index)
    }
}

impl<'a, T: Component> ComponentRef<'a> for &'a T where &'a T: ComponentBorrow<'a> {}

pub trait ComponentBorrowTuple<'a> {
    const READ_COMPONENT_SET: ComponentSet;
    const WRITE_COMPONENT_SET: ComponentSet;
    type ValueType: ComponentTuple;

    unsafe fn get_components(storage: &'a ArchetypeStorage, index: Index) -> Self;
}

impl<'a> ComponentBorrowTuple<'a> for () {
    type ValueType = ();

    const READ_COMPONENT_SET: ComponentSet = ComponentSet([0; COMPONENT_SET_LEN]);
    const WRITE_COMPONENT_SET: ComponentSet = ComponentSet([0; COMPONENT_SET_LEN]);

    unsafe fn get_components(storage: &'a ArchetypeStorage, index: Index) -> Self {
        ()
    }
}

pub trait ComponentRefTuple<'a>: ComponentBorrowTuple<'a> {}

impl<'a> ComponentRefTuple<'a> for () {}


component_tuple_impl!(10);
