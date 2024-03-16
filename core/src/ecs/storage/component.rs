use std::{
    fmt,
    fmt::{Debug, Error, Formatter},
    mem::MaybeUninit,
    ptr,
};

use crate::ecs::{component::Component, storage::sync_unsafe_cell::SyncUnsafeCell, Index};

pub trait ComponentStorage: Send + Sync {
    unsafe fn swap_remove_unchecked(&self, index: Index);
    unsafe fn drop_at_unchecked(&mut self, index: Index);
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
    pub data: Vec<SyncUnsafeCell<MaybeUninit<T>>>,
}

impl<T: Component> ComponentVec<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub unsafe fn get_unchecked(&self, index: Index) -> &T {
        (&*self.data.get_unchecked(index as usize).get()).assume_init_ref()
    }

    pub unsafe fn get_mut_unchecked(&self, index: Index) -> &mut T {
        (&mut *self.data.get_unchecked(index as usize).get()).assume_init_mut()
    }

    pub unsafe fn remove_unchecked(&self, index: Index) -> T {
        let value_ptr = (&mut *self.data.get_unchecked(index as usize).get()).as_mut_ptr();
        ptr::read(value_ptr)
    }

    pub unsafe fn insert_unchecked(&mut self, index: Index, value: T) {
        if (index as usize) >= self.data.len() {
            let diff = (index as usize + 1) - self.data.len();
            self.data.reserve(diff);
            self.data.set_len(index as usize + 1);
        }
        (&mut *self.data.get_unchecked_mut(index as usize).get()).write(value);
    }
}

impl<T: Component> ComponentStorage for ComponentVec<T> {
    unsafe fn swap_remove_unchecked(&self, index: Index) {
        todo!()
    }

    unsafe fn drop_at_unchecked(&mut self, index: Index) {
        let _ = self.remove_unchecked(index);
    }

    unsafe fn fmt_at_unchecked(&self, f: &mut Formatter<'_>, index: Index) -> Result<(), Error> {
        <dyn Debug>::fmt(self.get_unchecked(index), f)
    }
}
