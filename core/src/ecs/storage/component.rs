use std::{
    fmt,
    fmt::{Debug, Error, Formatter},
    mem::MaybeUninit,
    ptr,
};

use crate::ecs::{component::Component, storage::sync_unsafe_cell::SyncUnsafeCell, Index};

pub trait ComponentStorage: Send + Sync {
    fn swap_remove(&mut self, index: Index);
    unsafe fn fmt_at_unchecked(
        &self,
        f: &mut Formatter<'_>,
        index: Index,
    ) -> Result<(), fmt::Error>;
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

    pub unsafe fn get_unchecked(&self, index: Index) -> &T {
        &*self.data.get_unchecked(index as usize).get()
    }

    pub unsafe fn get_mut_unchecked(&self, index: Index) -> &mut T {
        &mut *self.data.get_unchecked(index as usize).get()
    }

    pub fn push(&mut self, value: T) -> Index {
        self.data.push(SyncUnsafeCell::new(value));
        (self.data.len() - 1) as Index
    }
}

impl<T: Component> ComponentStorage for ComponentVec<T> {
    fn swap_remove(&mut self, index: Index) {
        self.data.swap_remove(index as usize);
    }

    unsafe fn fmt_at_unchecked(&self, f: &mut Formatter<'_>, index: Index) -> Result<(), Error> {
        <dyn Debug>::fmt(self.get_unchecked(index), f)
    }
}
