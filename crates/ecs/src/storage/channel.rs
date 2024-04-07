use std::{cell::UnsafeCell, ptr};

use thread_local::ThreadLocal;

struct UnsafeVecCell<T>(UnsafeCell<Vec<T>>);

impl<T> UnsafeVecCell<T> {
    pub fn new() -> Self {
        Self(UnsafeCell::new(Vec::new()))
    }

    pub fn get(&self) -> *mut Vec<T> {
        self.0.get()
    }

    pub fn get_mut(&mut self) -> &mut Vec<T> {
        self.0.get_mut()
    }
}

pub struct Channel<T: 'static + Sized + Send + Sync> {
    messages: UnsafeVecCell<T>,
    tl_messages: ThreadLocal<UnsafeVecCell<T>>,
}

impl<T: 'static + Sized + Send + Sync> Channel<T> {
    pub fn new() -> Self {
        Self {
            messages: UnsafeVecCell::new(),
            tl_messages: ThreadLocal::new(),
        }
    }

    pub unsafe fn send(&self, message: T) {
        (&mut *self.messages.get()).push(message);
    }

    pub unsafe fn send_tl(&self, message: T) {
        (&mut *self.tl_messages.get_or(|| UnsafeVecCell::new()).get()).push(message)
    }

    pub unsafe fn sender(&self) -> Sender<'_, T> {
        Sender {
            vec_cell: &self.messages,
        }
    }

    pub unsafe fn sender_tl(&self) -> Sender<'_, T> {
        Sender {
            vec_cell: &self.tl_messages.get_or(|| UnsafeVecCell::new()),
        }
    }

    pub unsafe fn messages(&self) -> &[T] {
        (&*self.messages.get()).as_slice()
    }

    pub unsafe fn flush(&self) {
        (&mut *self.messages.get()).clear();
    }

    pub unsafe fn sync(&mut self) {
        self.tl_messages.iter_mut().for_each(|c| {
            let v = &mut *self.messages.get();
            let tlv = &mut *c.get();
            v.append(tlv);
        });
    }
}

pub struct Sender<'a, T: 'static + Sized + Send + Sync> {
    vec_cell: &'a UnsafeVecCell<T>,
}

impl<'a, T: 'static + Sized + Send + Sync> Sender<'a, T> {
    pub fn send(&mut self, event: T) {
        unsafe {
            (&mut *self.vec_cell.get()).push(event);
        }
    }
}
