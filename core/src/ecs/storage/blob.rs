use std::{
    alloc,
    alloc::Layout,
    any::type_name,
    cmp,
    fmt::{Debug, Formatter},
    marker::PhantomData,
    mem,
    ops::Add,
    ptr,
    ptr::NonNull,
};

pub struct BlobStorage {
    pub ptr: NonNull<u8>,
    pub cap: usize,
    pub layout: Layout,
    pub drop_fn: fn(*mut u8) -> (),
    pub swap_remove_fn: fn(*mut u8, *mut u8) -> (),
    pub as_debug_fn: fn(*mut u8) -> Box<dyn Debug>,
}

impl BlobStorage {
    pub fn new<T: 'static + Sized + Debug>() -> Self {
        Self {
            layout: Layout::new::<T>(),
            cap: if std::mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
            ptr: NonNull::dangling(),
            drop_fn: |x| unsafe { ptr::drop_in_place(x as *mut T) },
            swap_remove_fn: |x, y| unsafe {
                ptr::swap(x as *mut T, y as *mut T);
                ptr::drop_in_place(y as *mut T);
            },
            as_debug_fn: |x| Box::new(DebugBlobEntry::<T> { ptr: (x as *mut T) }),
        }
    }

    pub fn grow_exact(&mut self, amount: usize) {
        debug_assert!(self.layout.size() != 0, "capacity overflow");
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

    pub fn grow_amortized(&mut self, amount: usize) {
        self.grow_exact(cmp::max(amount, self.cap));
    }

    pub fn grow(&mut self) {
        self.grow_exact(cmp::max(self.cap, 1));
    }

    pub unsafe fn swap_remove(&mut self, index: usize, len: usize) {
        (self.swap_remove_fn)(
            self.ptr.as_ptr().add(index * self.layout.size()),
            self.ptr.as_ptr().add((len - 1) * self.layout.size()),
        );
    }

    pub unsafe fn manually_drop(&mut self, len: usize) {
        if self.cap != 0 && self.layout.size() != 0 {
            for i in 0..len {
                unsafe {
                    (self.drop_fn)(self.ptr.as_ptr().add(i * self.layout.size()));
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

pub struct DebugBlobEntry<T: Debug> {
    pub ptr: *mut T,
}

impl<T: Debug> Debug for DebugBlobEntry<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { T::fmt(&*self.ptr, f) }
    }
}
