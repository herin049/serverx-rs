use core::fmt::{Debug, Formatter};
use std::{any::TypeId, cmp, marker::PhantomData, ops::Range, slice};

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
use crate::storage::column::Column;

pub type Generation = u64;
pub type ArchetypeId = u32;
pub type ArchetypeIdx = u32;

pub struct Archetype {
    table: Table,
    entity_lookup: Slab<usize>,
    generation: Generation,
    id: ArchetypeId,
}

impl Archetype {
    pub fn new<T: ComponentTuple>(id: ArchetypeId) -> Self {
        let mut component_columns = T::columns();
        let mut columns = Vec::with_capacity(component_columns.as_ref().len());
        columns.push(Column::new::<Entity>());
        columns.extend(component_columns.into_iter());
        let mut component_type_ids = T::type_ids();
        let mut type_ids = Vec::with_capacity(component_type_ids.as_ref().len());
        type_ids.push(TypeId::of::<Entity>());
        type_ids.extend(component_type_ids.into_iter());
        unsafe {
            Self {
                table: Table::from_raw_parts(columns.into_boxed_slice(), type_ids.into_boxed_slice()),
                entity_lookup: Slab::new(),
                generation: 1,
                id
            }

        }
    }

    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    pub fn entities(&self) -> &[Entity] {
        unsafe {
            let column = self.table.column_unchecked(0);
            slice::from_raw_parts(column.as_ptr::<Entity>().cast_const(), self.table.len())
        }
    }

    pub fn type_ids(&self) -> &[TypeId] {
        self.table.type_ids()
    }

    pub fn index_of(&self, entity: Entity) -> Option<usize> {
        self.entity_lookup.get(entity.archetype_idx() as usize).map(|i| *i)
    }

    pub fn push<T: ComponentTuple>(&mut self, values: T) -> Entity {
        let table_len = self.table.len();
        unsafe {
            self.table.push(values);
        }
        let archetype_idx = self.entity_lookup.insert(table_len) as ArchetypeIdx;
        let entity = Entity::new(self.generation, self.id, archetype_idx);
        self.generation = self.generation.wrapping_add(1);
        unsafe {
            self.table.column(0).as_ptr::<Entity>().add(table_len).write(entity);
        }
        entity
    }

    pub fn get<'a, 'b, T: ComponentRefTuple<'b>>(&'a self, entity: Entity) -> Option<T>
    where
        'a: 'b,
    {
        if let Some(entity_idx) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if *self.entities().get_unchecked(*entity_idx) == entity {
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
                if self.entities().get_unchecked(*entity_idx).generation() == entity.generation() {
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
                if self.entities().get_unchecked(*i).generation() != entity.generation() {
                    return false;
                }
            }
            entity_idx = *i;
        } else {
            return false;
        }
        unsafe {
            *self.entity_lookup.get_unchecked_mut(
                self.entities()
                    .get_unchecked(self.table.len() - 1)
                    .archetype_idx() as usize,
            ) = entity_idx;
            self.table.swap_remove(entity_idx);
            self.entity_lookup.remove(entity.archetype_idx() as usize);
            true
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        if let Some(idx) = self.entity_lookup.get(entity.archetype_idx() as usize) {
            unsafe {
                if self.entities().get_unchecked(*idx).generation() == entity.generation() {
                    return true;
                }
            }
        }
        false
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
                if self.0.entities().get_unchecked(*entity_idx).generation() == entity.generation() {
                    if let Ok(ptr) = self.0.table.try_as_mut_ptr::<T::ValueType>() {
                        return Some(T::deref(ptr));
                    }
                }
            }
        }
        None
    }

    pub fn table(&self) -> &Table {
        &self.0.table
    }
}

pub struct DebugArchetypeEntry<'a> {
    pub(crate) archetype: &'a Archetype,
    pub(crate) index: usize,
}

impl<'a> Debug for DebugArchetypeEntry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        DebugTableEntry::fmt(&DebugTableEntry {
            table: &self.archetype.table,
            index: self.index,
        }, f)
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
