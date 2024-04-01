use alloc::alloc;
use std::{
    alloc::Layout,
    cmp,
    fmt::{Debug, Formatter},
    mem, ptr,
    ptr::NonNull,
};

pub struct Column {
    ptr: NonNull<u8>,
    cap: usize,
    layout: Layout,
    drop_fn: Option<fn(*mut u8) -> ()>,
    swap_fn: fn(*mut u8, *mut u8) -> (),
    swap_remove_fn: fn(*mut u8, *mut u8) -> (),
    as_debug_fn: fn(*mut u8) -> Box<dyn Debug>,
}

impl Column {
    pub fn new<T: 'static + Sized + Debug>() -> Self {
        Self {
            layout: Layout::new::<T>(),
            cap: if mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
            ptr: NonNull::dangling(),
            drop_fn: if mem::needs_drop::<T>() {
                Some(|x| unsafe {
                    ptr::drop_in_place(x as *mut T);
                })
            } else {
                None
            },
            swap_fn: |x, y| unsafe {
                ptr::swap(x as *mut T, y as *mut T);
            },
            swap_remove_fn: |x, y| unsafe {
                ptr::swap(x as *mut T, y as *mut T);
                ptr::drop_in_place(y as *mut T);
            },
            as_debug_fn: |x| Box::new(DebugColumnEntry::<T> { ptr: x as *mut T }),
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn as_ptr<T: 'static + Sized + Debug>(&self) -> *mut T {
        self.ptr.as_ptr() as *mut T
    }

    pub fn grow_exact(&mut self, amount: usize) {
        assert_ne!(self.layout.size(), 0, "capacity overflow");
        let cap = self.cap + amount;
        let layout =
            Layout::from_size_align(self.layout.size() * cap, self.layout.align()).unwrap();
        assert!(layout.size() <= isize::MAX as usize, "alloc too large");
        let ptr = if self.cap == 0 {
            unsafe { alloc::alloc(layout) }
        } else {
            unsafe {
                alloc::realloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align(self.layout.size() * self.cap, self.layout.align())
                        .unwrap(),
                    layout.size(),
                )
            }
        };
        self.ptr = match NonNull::new(ptr) {
            None => alloc::handle_alloc_error(layout),
            Some(p) => p,
        };
        self.cap = cap;
    }

    pub fn grow(&mut self) {
        self.grow_exact(cmp::max(self.cap, 1));
    }

    pub fn grow_amortized(&mut self, amount: usize) {
        self.grow_exact(cmp::max(amount, self.cap));
    }

    pub unsafe fn swap(&mut self, i: usize, j: usize) {
        (self.swap_fn)(
            self.ptr.as_ptr().add(i * self.layout.size()),
            self.ptr.as_ptr().add(j * self.layout.size()),
        );
    }

    pub unsafe fn swap_remove(&mut self, index: usize, len: usize) {
        (self.swap_remove_fn)(
            self.ptr.as_ptr().add(index * self.layout.size()),
            self.ptr.as_ptr().add((len - 1) * self.layout.size()),
        )
    }

    pub unsafe fn manually_drop(&mut self, len: usize) {
        if self.cap != 0 && self.layout.size() != 0 {
            if let Some(drop_fn) = self.drop_fn {
                for i in 0..len {
                    unsafe {
                        (drop_fn)(self.ptr.as_ptr().add(i * self.layout.size()));
                    }
                }
            }
            let layout =
                Layout::from_size_align(self.layout.size() * len, self.layout.align()).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_mut(), layout);
            }
        }
    }

    pub unsafe fn debug_entry(&self, index: usize) -> Box<dyn Debug> {
        (self.as_debug_fn)(self.ptr.as_ptr().add(index * self.layout.size()))
    }
}

pub struct DebugColumnEntry<T: Debug> {
    ptr: *mut T,
}

impl<T: Debug> Debug for DebugColumnEntry<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { T::fmt(&*self.ptr, f) }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::column::Column;

    #[test]
    fn test() {
        unsafe {
            let mut c = Column::new::<i32>();
            c.grow();
            c.as_ptr::<i32>().write(123i32);
            println!("{:?}", c.debug_entry(0));
            c.manually_drop(1);
        }
    }
}
