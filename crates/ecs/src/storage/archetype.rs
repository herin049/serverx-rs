use std::{
    alloc::Layout,
    any::TypeId,
    cmp,
    fmt::{Debug, DebugTuple, Formatter},
    marker::PhantomData,
    ops::{Index, Range},
    ptr,
};

use itertools::Itertools;
use slab::Slab;

use crate::{
    entity::Entity,
    sort::{insertion_cosort, insertion_sort},
    storage::blob::BlobStorage,
    tuple::{
        ptr::PtrTuple, type_tuple::TypeTuple, ComponentBorrowTuple, ComponentRefTuple,
        ComponentTuple,
    },
    ArchetypeId, ArchetypeIndex, Generation,
};

pub struct TryAsMutPtrError;

pub struct ArchetypeStorage {
    pub type_ids: Box<[TypeId]>,
    pub component_ptrs: Box<[*mut u8]>,
    pub entities: Vec<Entity>,
    pub entity_lookup: Slab<ArchetypeIndex>,
    pub archetype_id: ArchetypeId,
    pub generation: Generation,
    pub len: usize,
    pub cap: usize,
    pub components: Box<[BlobStorage]>,
}

impl ArchetypeStorage {
    pub fn new<T: TypeTuple + ComponentTuple>(archetype_id: ArchetypeId) -> Self {
        let mut type_ids = T::type_ids();
        let mut components = T::blobs();
        insertion_cosort(type_ids.as_mut(), components.as_mut());
        let component_ptrs: Box<[*mut u8]> =
            components.as_ref().iter().map(|b| b.ptr.as_ptr()).collect();
        Self {
            type_ids: type_ids.into(),
            component_ptrs,
            entities: Vec::new(),
            entity_lookup: Slab::new(),
            components: components.into(),
            generation: 0,
            len: 0,
            cap: 0,
            archetype_id,
        }
    }

    pub unsafe fn get<'a, T: ComponentRefTuple<'a>>(&'a self, index: usize) -> T {
        let ptrs = self.as_mut_ptr::<T::ValueType>().offset(index as isize);
        T::deref(ptrs)
    }

    pub unsafe fn get_mut<'a, T: ComponentBorrowTuple<'a>>(&'a self, index: usize) -> T {
        let ptrs = self.as_mut_ptr::<T::ValueType>().offset(index as isize);
        T::deref(ptrs)
    }

    pub unsafe fn swap_remove(&mut self, index: usize) {
        self.components.iter_mut().for_each(|b| {
            b.swap_remove(index, self.len);
        });
    }

    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(index) = self.entity_lookup.get(entity.archetype_index() as usize) {
            let last = *self.entities.last().unwrap();
            unsafe {
                self.components.iter_mut().for_each(|b| {
                    b.swap_remove(*index as usize, self.len);
                });
            }
            self.entities.swap_remove(*index as usize);
            unsafe {
                *self
                    .entity_lookup
                    .get_unchecked_mut(last.archetype_index() as usize) = *index;
            }
            self.entity_lookup.remove(entity.archetype_index() as usize);
            self.len -= 1;
        }
        false
    }

    pub unsafe fn push_entity<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let generation = self.generation;
        self.generation = self.generation.wrapping_add(1);
        let vacant_entry = self.entity_lookup.vacant_entry();
        let entity = Entity::new(
            generation,
            self.archetype_id,
            vacant_entry.key() as ArchetypeIndex,
        );
        vacant_entry.insert(self.len as ArchetypeIndex);
        self.entities.push(entity);
        self.push(values);
        entity
    }

    pub unsafe fn push<T: ComponentTuple>(&mut self, values: T) {
        if self.len >= self.cap {
            let grow = cmp::max(self.len - self.cap + 1, self.cap);
            self.components.iter_mut().enumerate().for_each(|(i, b)| {
                if b.cap < self.cap + grow {
                    b.grow_exact(grow);
                    *self.component_ptrs.get_unchecked_mut(i) = b.ptr.as_ptr();
                }
            });
            self.cap += grow;
        }
        let ptr = T::PtrType::offset(self.as_mut_ptr::<T>(), self.len as isize);
        self.len += 1;
        T::write(values, ptr);
    }

    pub fn try_as_mut_ptr<T: ComponentTuple>(&self) -> Result<T::PtrType, TryAsMutPtrError> {
        let type_ids = T::type_ids();
        let mut ptrs = T::PtrType::null_ptr_slice();
        let ptrs_len = ptrs.as_ref().len();
        for i in 0..ptrs_len {
            let mut found = false;
            for j in 0..self.type_ids.len() {
                unsafe {
                    if *type_ids.as_ref().get_unchecked(i) == *self.type_ids.get_unchecked(j) {
                        found = true;
                        *ptrs.as_mut().get_unchecked_mut(i) = *self.component_ptrs.get_unchecked(j);
                        break;
                    }
                }
            }
            if !found {
                return Err(TryAsMutPtrError);
            }
        }
        Ok(T::PtrType::from_ptr_slice(ptrs.as_ref()))
    }

    pub fn as_mut_ptr<T: ComponentTuple>(&self) -> T::PtrType {
        let type_ids = T::type_ids();
        let mut ptrs = T::PtrType::null_ptr_slice();
        let ptrs_len = ptrs.as_ref().len();
        for i in 0..ptrs_len {
            let mut found = false;
            for j in 0..self.type_ids.len() {
                unsafe {
                    if *type_ids.as_ref().get_unchecked(i) == *self.type_ids.get_unchecked(j) {
                        found = true;
                        *ptrs.as_mut().get_unchecked_mut(i) = *self.component_ptrs.get_unchecked(j);
                        break;
                    }
                }
            }
            if !found {
                panic!("component not in component set");
            }
        }
        T::PtrType::from_ptr_slice(ptrs.as_ref())
    }

    pub unsafe fn iter<'a, 'b, T: ComponentRefTuple<'b> + ComponentBorrowTuple<'b>>(
        &'a self,
    ) -> ArchetypeIter<'_, 'b, T> where 'a: 'b {
        ArchetypeIter {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.as_mut_ptr::<<T as ComponentBorrowTuple<'b>>::ValueType>(),
            curr: 0,
            end: self.len,
        }
    }

    pub unsafe fn iter_mut<'a, 'b, T: ComponentBorrowTuple<'b>>(&'a self) -> ArchetypeIter<'_, 'b, T> {
        ArchetypeIter {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.as_mut_ptr::<<T as ComponentBorrowTuple<'b>>::ValueType>(),
            curr: 0,
            end: self.len,
        }
    }

    pub unsafe fn chunks<'a, 'b, T: ComponentRefTuple<'b> + ComponentBorrowTuple<'b>>(
        &'a self,
        chunk_size: usize,
    ) -> ArchetypeChunks<'_, 'b, T> where 'a: 'b {
        ArchetypeChunks {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.as_mut_ptr::<<T as ComponentBorrowTuple<'b>>::ValueType>(),
            curr: 0,
            end: self.len,
            chunk_size,
        }
    }

    pub unsafe fn chunks_mut<'a, 'b, T: ComponentBorrowTuple<'b>>(
        &self,
        chunk_size: usize,
    ) -> ArchetypeChunks<'_, 'b, T> {
        ArchetypeChunks {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.len,
            chunk_size,
        }
    }
}

pub struct ArchetypeIter<'a, 'b, T: ComponentBorrowTuple<'b>> where 'a: 'b {
    phantom: PhantomData<&'a ArchetypeStorage>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ComponentTuple>::PtrType,
    curr: usize,
    end: usize,
}

unsafe impl Send for ArchetypeStorage {}
unsafe impl Sync for ArchetypeStorage {}

impl<'a, 'b, T: ComponentBorrowTuple<'b>> Iterator for ArchetypeIter<'a, 'b, T> {
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.end {
            let entity = unsafe { *self.entities.get_unchecked(self.curr) };
            let values = unsafe { T::deref(self.ptr.offset(self.curr as isize)) };
            self.curr += 1;
            Some((entity, values))
        } else {
            None
        }
    }
}

pub struct ArchetypeChunks<'a, 'b, T: ComponentBorrowTuple<'b>> {
    phantom: PhantomData<&'a ArchetypeStorage>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ComponentTuple>::PtrType,
    curr: usize,
    end: usize,
    chunk_size: usize,
}

impl<'a, 'b, T: ComponentBorrowTuple<'b>> Iterator for ArchetypeChunks<'a, 'b, T> {
    type Item = ArchetypeChunk<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.end {
            let chunk_size = cmp::min(self.end - self.curr, self.chunk_size);
            let next = ArchetypeChunk {
                phantom: PhantomData,
                entities: &self.entities,
                ptr: self.ptr,
                curr: self.curr,
                end: self.curr + chunk_size,
            };
            self.curr += chunk_size;
            Some(next)
        } else {
            None
        }
    }
}

pub struct ArchetypeChunk<'a, 'b, T: ComponentBorrowTuple<'b>> {
    phantom: PhantomData<&'a ArchetypeStorage>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ComponentTuple>::PtrType,
    curr: usize,
    end: usize,
}

unsafe impl<'a, 'b, T: ComponentBorrowTuple<'b>> Sync for ArchetypeChunk<'a, 'b, T> where T::ValueType: Sync {}
unsafe impl<'a, 'b, T: ComponentBorrowTuple<'b>> Send for ArchetypeChunk<'a, 'b, T> where T::ValueType: Sync {}

impl<'a, 'b, T: ComponentBorrowTuple<'b>> Iterator for ArchetypeChunk<'a, 'b, T> {
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.end {
            let entity = unsafe { *self.entities.get_unchecked(self.curr) };
            let values = unsafe { T::deref(self.ptr.offset(self.curr as isize)) };
            self.curr += 1;
            Some((entity, values))
        } else {
            None
        }
    }
}

impl Drop for ArchetypeStorage {
    fn drop(&mut self) {
        self.components
            .iter_mut()
            .for_each(|b| unsafe { b.manually_drop(self.len) });
    }
}

pub struct DebugArchetypeEntry<'a> {
    pub(crate) archetype_storage: &'a ArchetypeStorage,
    pub(crate) index: usize,
}

impl<'a> Debug for DebugArchetypeEntry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_tuple = f.debug_tuple("ArchetypeEntry");
        self.archetype_storage
            .components
            .iter()
            .for_each(|b| unsafe {
                debug_tuple.field(&b.debug_entry(self.index));
            });
        debug_tuple.finish()
    }
}

impl Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        (0..self.len).for_each(|i| {
            debug_list.entry(&DebugArchetypeEntry {
                archetype_storage: self,
                index: i,
            });
        });
        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity::Entity, storage::archetype::ArchetypeStorage};

    #[derive(Debug)]
    pub struct Position(i32);

    #[derive(Debug)]
    pub struct Velocity(i64);

    #[derive(Debug)]
    pub struct Health(f32);

    #[derive(Debug)]
    pub struct Foo;

    #[test]
    fn test() {
        unsafe {
            let mut a = ArchetypeStorage::new::<(i32, i64, f32, Foo)>(0);
            println!("{:?}", a);
            a.push_entity((1i32, 2i64, 3.14f32, Foo));
            a.push_entity((4i32, 5i64, 6.14f32, Foo));
            a.push_entity((7i32, 8i64, 9.14f32, Foo));
            a.push_entity((10i32, 11i64, 12.14f32, Foo));
            a.push_entity((10i32, 11i64, 12.14f32, Foo));
            a.push_entity((10i32, 11i64, 12.14f32, Foo));
            a.push_entity((10i32, 11i64, 12.14f32, Foo));
            a.push_entity((10i32, 11i64, 12.14f32, Foo));
            println!("{:#?}", a);
            for chunk in a.chunks_mut::<(&i32, &mut i64, &f32, &Foo)>(3) {
                println!("chunk");
                for (entity, (a, b, c, d)) in chunk {
                    println!("{:?} {:?} {:?} {:?}", a, b, c, d);
                    *b *= 2;
                }
            }
            for (entity, (a, b, c, d)) in a.iter_mut::<(&i32, &mut i64, &f32, &Foo)>() {
                println!("entity {} {} {} {:?}", a, b, c, d);
            }
            println!("{:#?}", a);
        }
    }
}
