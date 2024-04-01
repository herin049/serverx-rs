use core::fmt::{Debug, Formatter};
use std::{any::TypeId, cmp, marker::PhantomData, ops::Range};

use slab::Slab;

use crate::{
    entity::Entity,
    storage::table::{DebugTableEntry, Table},
    tuple::{
        borrow::{BorrowTuple, RefTuple},
        component::{ComponentBorrowTuple, ComponentRefTuple, ComponentTuple},
        ptr::PtrTuple,
        value::ValueTuple,
    },
};

pub type Generation = u64;
pub type ArchetypeId = u32;
pub type ArchetypeIdx = u32;

pub struct Archetype {
    table: Table,
    entities: Vec<Entity>,
    entity_lookup: Slab<usize>,
    generation: Generation,
    id: ArchetypeId,
}

impl Archetype {
    pub fn new<T: ComponentTuple>(id: ArchetypeId) -> Self {
        Self {
            table: Table::new::<T>(),
            entities: Vec::new(),
            entity_lookup: Slab::new(),
            generation: 1,
            id,
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    pub fn entities(&self) -> &[Entity] {
        self.entities.as_slice()
    }

    pub fn type_ids(&self) -> &[TypeId] {
        self.table.type_ids()
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        unsafe {
            self.table.push(values);
        }
        let archetype_idx = self.entity_lookup.insert(self.entities.len()) as ArchetypeIdx;
        let entity = Entity::new(self.generation, self.id, archetype_idx);
        self.entities.push(entity);
        entity
    }

    pub fn get<'a, 'b, T: ComponentRefTuple<'b>>(&'a self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        if let Some(entity_idx) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.entities.get_unchecked(*entity_idx).generation() == entity.generation() {
                    if let Ok(ptr) = self.table.try_as_mut_ptr::<T::ValueType>() {
                        return Some(T::deref(ptr));
                    }
                }
            }
        }
        None
    }

    pub fn get_mut<'a, 'b, T: ComponentBorrowTuple<'b>>(&'a mut self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        if let Some(entity_idx) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.entities.get_unchecked(*entity_idx).generation() == entity.generation() {
                    if let Ok(ptr) = self.table.try_as_mut_ptr::<T::ValueType>() {
                        return Some(T::deref(ptr));
                    }
                }
            }
        }
        None
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        let mut entity_idx = 0;
        if let Some(i) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.entities.get_unchecked(*i).generation() != entity.generation() {
                    return false;
                }
            }
            entity_idx = *i;
        } else {
            return false;
        }
        unsafe {
            *self.entity_lookup.get_unchecked_mut(
                self.entities
                    .get_unchecked(self.entities.len() - 1)
                    .archetype_idx() as usize,
            ) = entity_idx;
            self.table.swap_remove(entity_idx);
            self.entities.swap_remove(entity_idx);
            self.entity_lookup.remove(entity.archetype_idx() as usize);
            true
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        if let Some(idx) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.entities.get_unchecked(*idx).generation() == entity.generation() {
                    return true;
                }
            }
        }
        false
    }

    pub fn partitions<'a, 'b, 'c, T: RefTuple<'c>>(
        &'a self,
        partition_size: usize,
    ) -> ArchetypePartitions<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypePartitions {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            size: partition_size,
            curr: 0,
            end: self.entities.len(),
        }
    }

    pub fn partitions_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(
        &'a mut self,
        partition_size: usize,
    ) -> ArchetypePartitionsMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypePartitionsMut {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            size: partition_size,
            curr: 0,
            end: self.entities.len(),
        }
    }

    pub fn iter<'a, 'b, 'c, T: RefTuple<'c>>(&'a self) -> ArchetypeIter<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIter {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.entities.len(),
        }
    }

    pub fn iter_range<'a, 'b, 'c, T: RefTuple<'c>>(
        &'a self,
        range: Range<usize>,
    ) -> ArchetypeIter<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIter {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            curr: cmp::min(range.start, self.entities.len()),
            end: cmp::min(range.end, self.entities.len()),
        }
    }

    pub fn iter_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(&'a mut self) -> ArchetypeIterMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIterMut {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.entities.len(),
        }
    }

    pub fn iter_range_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(
        &'a mut self,
        range: Range<usize>,
    ) -> ArchetypeIterMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIterMut {
            phantom: PhantomData,
            entities: self.entities.as_slice(),
            ptr: self.table.as_mut_ptr::<T::ValueType>(),
            curr: cmp::min(range.start, self.entities.len()),
            end: cmp::min(range.end, self.entities.len()),
        }
    }
}

pub struct UnsafeArchetypeCell<'a>(pub &'a Archetype);

unsafe impl<'a> Send for UnsafeArchetypeCell<'a> {}
unsafe impl<'a> Sync for UnsafeArchetypeCell<'a> {}

impl<'a> UnsafeArchetypeCell<'a> {
    pub unsafe fn get<'b, T: ComponentRefTuple<'b>>(&self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        self.0.get::<'a, 'b, T>(entity)
    }

    pub unsafe fn get_mut<'b, T: ComponentBorrowTuple<'b>>(&self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        if let Some(entity_idx) = self.0.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.0.entities.get_unchecked(*entity_idx).generation() == entity.generation() {
                    if let Ok(ptr) = self.0.table.try_as_mut_ptr::<T::ValueType>() {
                        return Some(T::deref(ptr));
                    }
                }
            }
        }
        None
    }

    pub unsafe fn partitions<'b, 'c, T: RefTuple<'c>>(
        &self,
        partition_size: usize,
    ) -> ArchetypePartitions<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        self.0.partitions::<'a, 'b, 'c, T>(partition_size)
    }

    pub unsafe fn partitions_mut<'b, 'c, T: BorrowTuple<'c>>(
        &self,
        partition_size: usize,
    ) -> ArchetypePartitionsMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypePartitionsMut {
            phantom: PhantomData,
            entities: self.0.entities.as_slice(),
            ptr: self.0.table.as_mut_ptr::<T::ValueType>(),
            size: partition_size,
            curr: 0,
            end: self.0.entities.len(),
        }
    }

    pub unsafe fn iter<'b, 'c, T: RefTuple<'c>>(&self) -> ArchetypeIter<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        self.0.iter::<'a, 'b, 'c, T>()
    }

    pub unsafe fn iter_range<'b, 'c, T: RefTuple<'c>>(
        &self,
        range: Range<usize>,
    ) -> ArchetypeIter<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        self.0.iter_range::<'a, 'b, 'c, T>(range)
    }

    pub unsafe fn iter_mut<'b, 'c, T: BorrowTuple<'c>>(&self) -> ArchetypeIterMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIterMut {
            phantom: PhantomData,
            entities: self.0.entities.as_slice(),
            ptr: self.0.table.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.0.entities.len(),
        }
    }

    pub fn iter_range_mut<'b, 'c, T: BorrowTuple<'c>>(
        &self,
        range: Range<usize>,
    ) -> ArchetypeIterMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        ArchetypeIterMut {
            phantom: PhantomData,
            entities: self.0.entities.as_slice(),
            ptr: self.0.table.as_mut_ptr::<T::ValueType>(),
            curr: cmp::min(range.start, self.0.entities.len()),
            end: cmp::min(range.end, self.0.entities.len()),
        }
    }
}

pub struct ArchetypePartitions<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    size: usize,
    curr: usize,
    end: usize,
}

unsafe impl<'a, 'b, T: RefTuple<'b>> Send for ArchetypePartitions<'a, 'b, T> where T: Send {}

impl<'a, 'b, T: RefTuple<'b>> Iterator for ArchetypePartitions<'a, 'b, T>
where
    'a: 'b,
{
    type Item = ArchetypePartition<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            Some(ArchetypePartition {
                phantom: PhantomData,
                entities: &self.entities,
                ptr: self.ptr,
                start: self.curr,
                end: cmp::min(self.curr + self.size, self.end),
            })
        } else {
            None
        };
        self.curr += self.size;
        result
    }
}

pub struct ArchetypePartitionsMut<'a, 'b, T: BorrowTuple<'b>> {
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    size: usize,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: BorrowTuple<'b>> ArchetypePartitionsMut<'a, 'b, T> {
    pub fn empty() -> Self {
        Self {
            phantom: PhantomData,
            entities: &[],
            ptr: <T::ValueType as ValueTuple>::PtrType::null_ptr(),
            size: 0,
            curr: 0,
            end: 0,
        }
    }
}

unsafe impl<'a, 'b, T: BorrowTuple<'b>> Send for ArchetypePartitionsMut<'a, 'b, T> where T: Send {}

impl<'a, 'b, T: BorrowTuple<'b>> Iterator for ArchetypePartitionsMut<'a, 'b, T>
where
    'a: 'b,
{
    type Item = ArchetypePartitionMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            Some(ArchetypePartitionMut {
                phantom: PhantomData,
                entities: &self.entities,
                ptr: self.ptr,
                start: self.curr,
                end: cmp::min(self.curr + self.size, self.end),
            })
        } else {
            None
        };
        self.curr += self.size;
        result
    }
}

pub struct ArchetypePartition<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    start: usize,
    end: usize,
}

unsafe impl<'a, 'b, T: RefTuple<'b>> Send for ArchetypePartition<'a, 'b, T> where T: Send {}

impl<'a, 'b, T: RefTuple<'b>> ArchetypePartition<'a, 'b, T>
where
    'a: 'b,
{
    pub fn iter<'c>(&self) -> ArchetypeIter<'c, 'b, T>
    where
        'c: 'b,
        'a: 'c,
    {
        ArchetypeIter {
            phantom: PhantomData,
            entities: &self.entities,
            ptr: self.ptr,
            curr: self.start,
            end: self.end,
        }
    }
}

pub struct ArchetypePartitionMut<'a, 'b, T: BorrowTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    start: usize,
    end: usize,
}

unsafe impl<'a, 'b, T: BorrowTuple<'b>> Send for ArchetypePartitionMut<'a, 'b, T> where T: Send {}

impl<'a, 'b, T: BorrowTuple<'b>> ArchetypePartitionMut<'a, 'b, T>
where
    'a: 'b,
{
    pub fn iter<'c>(&mut self) -> ArchetypeIterMut<'c, 'b, T>
    where
        'c: 'b,
        'a: 'c,
    {
        ArchetypeIterMut {
            phantom: PhantomData,
            entities: &self.entities,
            ptr: self.ptr,
            curr: self.start,
            end: self.end,
        }
    }
}

pub struct ArchetypeIter<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: RefTuple<'b>> Iterator for ArchetypeIter<'a, 'b, T>
where
    'a: 'b,
{
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            unsafe {
                let entity = *self.entities.get_unchecked(self.curr);
                let values = T::deref(self.ptr.add(self.curr));
                Some((entity, values))
            }
        } else {
            None
        };
        self.curr += 1;
        result
    }
}

pub struct ArchetypeIterMut<'a, 'b, T: BorrowTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(UnsafeArchetypeCell<'a>, T)>,
    entities: &'a [Entity],
    ptr: <T::ValueType as ValueTuple>::PtrType,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: BorrowTuple<'b>> Iterator for ArchetypeIterMut<'a, 'b, T>
where
    'a: 'b,
{
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            unsafe {
                let entity = *self.entities.get_unchecked(self.curr);
                let values = T::deref(self.ptr.add(self.curr));
                Some((entity, values))
            }
        } else {
            None
        };
        self.curr += 1;
        result
    }
}

pub struct DebugArchetypeEntry<'a> {
    pub(crate) archetype: &'a Archetype,
    pub(crate) index: usize,
}

impl<'a> Debug for DebugArchetypeEntry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug_tuple = f.debug_tuple("ArchetypeEntry");
        debug_tuple.field(self.archetype.entities.get(self.index).unwrap());
        debug_tuple.field(&DebugTableEntry {
            table: &self.archetype.table,
            index: self.index,
        });
        debug_tuple.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{archetype::Archetype, component::Component};

    #[derive(Debug)]
    pub struct ComponentA(i64);
    #[derive(Debug)]
    pub struct ComponentB(i64);
    #[derive(Debug)]
    pub struct ComponentC(i64);

    impl Component for ComponentA {}
    impl Component for ComponentB {}
    impl Component for ComponentC {}

    #[test]
    fn test() {
        let a = Archetype::new::<(ComponentA, ComponentB, ComponentC)>(0);
    }
}
