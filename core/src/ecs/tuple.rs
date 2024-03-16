use std::mem::MaybeUninit;

use serverx_macros::component_tuple_impl;

use crate::ecs::{
    component::{Component, ComponentSet, COMPONENT_SET_LEN},
    storage::{archetype::ArchetypeStorage, component::ComponentVec},
    Index,
};

pub trait ComponentTuple {
    const COMPONENT_SET: ComponentSet;
    const COMPONENT_COUNT: usize;

    unsafe fn init_storage_unchecked(storage: &mut ArchetypeStorage);
    unsafe fn insert_unchecked(self, storage: &mut ArchetypeStorage, index: Index);
}

pub trait ComponentBorrow<'a> {
    type ValueType: Component;
    const REF: bool;
    const MUT: bool;

    unsafe fn get_unchecked(storage: &'a ArchetypeStorage, index: Index) -> Self;
}

pub trait ComponentRef<'a>: ComponentBorrow<'a> {}

impl<'a, T: Component> ComponentBorrow<'a> for &'a T {
    type ValueType = T;

    const MUT: bool = false;
    const REF: bool = true;

    unsafe fn get_unchecked(storage: &'a ArchetypeStorage, index: Index) -> Self {
        storage
            .components
            .get_unchecked(Self::ValueType::ID as usize)
            .assume_init_ref()
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_unchecked(index)
    }
}

impl<'a, T: Component> ComponentBorrow<'a> for &'a mut T {
    type ValueType = T;

    const MUT: bool = true;
    const REF: bool = false;

    unsafe fn get_unchecked(storage: &'a ArchetypeStorage, index: Index) -> Self {
        storage
            .components
            .get_unchecked(Self::ValueType::ID as usize)
            .assume_init_ref()
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_mut_unchecked(index)
    }
}

impl<'a, T: Component> ComponentRef<'a> for &'a T where &'a T: ComponentBorrow<'a> {}

pub trait ComponentBorrowTuple<'a> {
    const READ_COMPONENT_SET: ComponentSet;
    const WRITE_COMPONENT_SET: ComponentSet;
    type ValueType: ComponentTuple;

    unsafe fn get_unchecked(storage: &'a ArchetypeStorage, index: Index) -> Self;
}

pub trait ComponentRefTuple<'a>: ComponentBorrowTuple<'a> {}

component_tuple_impl!(10);
