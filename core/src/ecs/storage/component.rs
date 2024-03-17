use std::{
    fmt,
    fmt::{Debug, Error, Formatter},
    mem::MaybeUninit,
    ptr,
};

use itertools::Itertools;

use crate::ecs::{component::Component, storage::sync_unsafe_cell::SyncUnsafeCell, Index};

pub trait ComponentStorage: Debug + Send + Sync {
    fn swap_remove_component(&mut self, index: Index);
    fn get_dyn_component(&self, index: Index) -> &dyn Debug;
}

impl dyn ComponentStorage {
    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: ComponentStorage>(&self) -> &T {
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const Self as *const T) }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: ComponentStorage>(&mut self) -> &mut T {
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &mut *(self as *mut Self as *mut T) }
    }
}

pub struct ComponentVec<T: Component> {
    pub data: Vec<SyncUnsafeCell<T>>,
}

impl<T: Component> ComponentVec<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub unsafe fn get_component_unchecked(&self, index: Index) -> &T {
        &*self.data.get_unchecked(index as usize).get()
    }

    pub unsafe fn get_mut_component_unchecked(&self, index: Index) -> &mut T {
        &mut *self.data.get_unchecked(index as usize).get()
    }

    #[inline(always)]
    pub fn push_component(&mut self, value: T) -> Index {
        self.data.push(SyncUnsafeCell::new(value));
        (self.data.len() - 1) as Index
    }
}

impl<T: Component> Debug for ComponentVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}]",
            self.data
                .iter()
                .map(|e| { unsafe { &*e.get() } })
                .format(", ")
        )
    }
}

impl<T: Component> ComponentStorage for ComponentVec<T> {
    fn swap_remove_component(&mut self, index: Index) {
        self.data.swap_remove(index as usize);
    }

    fn get_dyn_component(&self, index: Index) -> &dyn Debug {
        unsafe { &*self.data[index as usize].get() }
    }
}
