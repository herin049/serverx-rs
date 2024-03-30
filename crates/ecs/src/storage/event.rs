use std::{
    any::{Any, TypeId},
    cell::{RefCell, UnsafeCell},
    collections::BTreeMap,
};

use thread_local::ThreadLocal;

use crate::event::Event;

pub struct EventGroup<T: Event> {
    events: UnsafeCell<Vec<T>>,
    queued: ThreadLocal<UnsafeCell<Vec<T>>>,
}

impl<T: Event> EventGroup<T> {
    pub fn new() -> Self {
        Self {
            events: UnsafeCell::new(Vec::new()),
            queued: ThreadLocal::new(),
        }
    }

    pub fn sync(&mut self) {
        self.queued.iter_mut().for_each(|v| {
            let vec = v.get_mut();
            vec.drain(0..vec.len()).for_each(|e| {
                self.events.get_mut().push(e);
            });
        });
    }

    pub fn flush(&mut self) {
        self.events.get_mut().clear();
    }
}

pub struct EventStorage {
    groups: BTreeMap<TypeId, Box<dyn Any>>,
}

unsafe impl Sync for EventStorage {}
unsafe impl Send for EventStorage {}

impl EventStorage {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }

    pub fn register<T: Event>(&mut self) {
        if self.groups.contains_key(&TypeId::of::<T>()) {
            return;
        }
        self.groups
            .insert(TypeId::of::<T>(), Box::new(EventGroup::<T>::new()));
    }

    pub fn sync<T: Event>(&mut self) {
        if let Some(g) = self.groups.get_mut(&TypeId::of::<T>()) {
            unsafe {
                let group = &mut *(g.as_mut() as *mut dyn Any as *mut EventGroup<T>);
                group.sync();
            }
        }
    }

    pub fn flush<T: Event>(&mut self) {
        if let Some(g) = self.groups.get_mut(&TypeId::of::<T>()) {
            unsafe {
                let group = &mut *(g.as_mut() as *mut dyn Any as *mut EventGroup<T>);
                group.flush();
            }
        }
    }

    pub unsafe fn events<'a, T: Event>(&'a self) -> &'a [T] {
        if let Some(g) = self.groups.get(&TypeId::of::<T>()) {
            let group = &*(g.as_ref() as *const dyn Any as *const EventGroup<T>);
            (&*group.events.get()).as_slice()
        } else {
            &[]
        }
    }

    pub unsafe fn send<'a, T: Event>(&'a self, e: T) {
        if let Some(g) = self.groups.get(&TypeId::of::<T>()) {
            let group = &*(g.as_ref() as *const dyn Any as *const EventGroup<T>);
            (&mut *group.events.get()).push(e);
        }
    }

    pub unsafe fn send_sync<'a, T: Event>(&'a self, e: T) {
        if let Some(g) = self.groups.get(&TypeId::of::<T>()) {
            let group = &*(g.as_ref() as *const dyn Any as *const EventGroup<T>);
            unsafe {
                (&mut *group.queued.get_or_default().get()).push(e);
            }
        }
    }
}
