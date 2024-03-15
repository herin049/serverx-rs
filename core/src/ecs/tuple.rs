use serverx_macros::component_tuple_impl;

use crate::ecs::{
    component::{Component, ComponentSet, COMPONENT_SET_LEN},
    storage::{sync_unsafe_cell::SyncUnsafeCell, ComponentStorage, ComponentVec},
    Index,
};

pub trait ComponentTuple {
    type OptionType;
    const COMPONENT_SET: ComponentSet;
    const COMPONENT_COUNT: usize;

    unsafe fn insert_unchecked(
        self,
        components: &Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>,
        index: Index,
    );

    unsafe fn put_unchecked(
        self,
        components: &Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>,
        component_set: &ComponentSet,
        index: Index,
    ) -> Self::OptionType;
}

pub trait ComponentBorrow {
    type ValueType: Component;
    const REF: bool;
    const MUT: bool;

    unsafe fn get_unchecked(
        storage: &SyncUnsafeCell<Box<dyn ComponentStorage>>,
        index: Index,
    ) -> Self;
}

pub trait ComponentRef: ComponentBorrow {}

impl<T: Component> ComponentRef for &T where for<'a> &'a T: ComponentBorrow {}

impl<T: Component> ComponentBorrow for &T {
    type ValueType = T;

    const MUT: bool = false;
    const REF: bool = true;

    unsafe fn get_unchecked(
        storage: &SyncUnsafeCell<Box<dyn ComponentStorage>>,
        index: Index,
    ) -> Self {
        (&*storage.get())
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_unchecked(index)
    }
}

impl<T: Component> ComponentBorrow for &mut T {
    type ValueType = T;

    const MUT: bool = true;
    const REF: bool = false;

    unsafe fn get_unchecked(
        storage: &SyncUnsafeCell<Box<dyn ComponentStorage>>,
        index: Index,
    ) -> Self {
        (&*storage.get())
            .downcast_ref_unchecked::<ComponentVec<T>>()
            .get_mut_unchecked(index)
    }
}

pub trait ComponentBorrowTuple {
    const READ_COMPONENT_SET: ComponentSet;
    const WRITE_COMPONENT_SET: ComponentSet;
    type ValueType: ComponentTuple;

    unsafe fn get_unchecked(
        storage: &SyncUnsafeCell<Vec<SyncUnsafeCell<Box<dyn ComponentStorage>>>>,
        index: Index,
    ) -> Self;
}

pub trait ComponentRefTuple: ComponentBorrowTuple {}

component_tuple_impl!(10);

pub struct Foo;

impl Foo {
    const BAR: ComponentSet = {
        let mut s = [0u64; COMPONENT_SET_LEN];
        s[0] = 23;
        s[1] = 34;
        ComponentSet(s)
    };
}

#[cfg(test)]
pub mod tests {
    use crate::ecs::tuple::Foo;

    #[test]
    fn test() {
        println!("{:?}", Foo::BAR);
    }
}
