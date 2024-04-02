use core::fmt::{Debug, Formatter};
use std::{any::TypeId, cmp, marker::PhantomData};
use std::ops::Range;

use crate::{
    storage::column::Column,
    tuple::{
        borrow::{BorrowTuple, RefTuple},
        ptr::PtrTuple,
        table::TableLayout,
        value::ValueTuple,
    },
};

pub struct Table {
    len: usize,
    column_ptrs: Box<[*mut u8]>,
    cap: usize,
    columns: Box<[Column]>,
    type_ids: Box<[TypeId]>,
}

#[derive(Copy, Clone, Debug)]
pub struct TryAsPtrError;

impl Table {
    pub unsafe fn from_raw_parts(columns: Box<[Column]>, type_ids: Box<[TypeId]>)  -> Self {
        let mut column_ptrs = Vec::with_capacity(columns.len());
        for c in columns.iter() {
            column_ptrs.push(c.as_ptr() as *mut u8);
        }
        Self {
            column_ptrs: column_ptrs.into_boxed_slice(),
            len: 0,
            cap: 0,
            columns,
            type_ids
        }
    }

    pub fn new<L: TableLayout>() -> Self {
        let columns = L::columns().into();
        let type_ids = L::type_ids().into();
        let mut column_ptrs = Vec::with_capacity(columns.len());
        for c in columns.iter() {
            column_ptrs.push(c.as_ptr() as *mut u8);
        }
        Self {
            column_ptrs: column_ptrs.into_boxed_slice(),
            len: 0,
            cap: 0,
            columns: L::columns().into(),
            type_ids: L::type_ids().into(),
        }
    }


    pub fn try_as_mut_ptr<T: ValueTuple>(&self) -> Result<T::PtrType, TryAsPtrError> {
        let type_ids = T::type_ids();
        let mut ptrs = T::PtrType::null_ptr_slice();
        'outer: for i in 0..ptrs.as_ref().len() {
            for j in 0..self.type_ids.len() {
                unsafe {
                    if *type_ids.as_ref().get_unchecked(i) == *self.type_ids.get_unchecked(j) {
                        *ptrs.as_mut().get_unchecked_mut(i) = *self.column_ptrs.get_unchecked(j);
                        continue 'outer;
                    }
                }
            }
            return Err(TryAsPtrError);
        }
        unsafe { Ok(T::PtrType::from_ptr_slice(ptrs.as_ref())) }
    }

    pub fn as_mut_ptr<T: ValueTuple>(&self) -> T::PtrType {
        self.try_as_mut_ptr::<T>().expect("type not in table")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn column(&self, column_idx: usize) -> &Column {
        &self.columns[column_idx]
    }

    pub unsafe fn column_unchecked(&self, column_idx: usize) -> &Column {
        self.columns.get_unchecked(column_idx)
    }

    pub fn type_ids(&self) -> &[TypeId] {
        self.type_ids.as_ref()
    }

    pub unsafe fn push<T: ValueTuple>(&mut self, values: T) {
        if self.len >= self.cap {
            let grow = cmp::max(self.len - self.cap + 1, self.cap);
            self.columns.iter_mut().enumerate().for_each(|(i, c)| {
                if c.capacity() < self.cap + grow {
                    c.grow_exact(grow);
                }
                *self.column_ptrs.get_unchecked_mut(i) = c.as_ptr();
            });
            self.cap += grow;
        }
        let ptr = T::PtrType::add(self.as_mut_ptr::<T>(), self.len);
        self.len += 1;
        T::write(values, ptr);
    }

    pub unsafe fn swap_remove(&mut self, index: usize) {
        self.columns.iter_mut().for_each(|x| {
            x.swap_remove(index, self.len);
        });
    }

    pub unsafe fn get<'a, 'b, T: RefTuple<'b>>(&'a self) -> T
    where
        'a: 'b,
    {
        T::deref(self.as_mut_ptr::<T::ValueType>())
    }

    pub unsafe fn get_mut<'a, 'b, T: BorrowTuple<'b>>(&'a self) -> T
    where
        'a: 'b,
    {
        T::deref(self.as_mut_ptr::<T::ValueType>())
    }

    pub unsafe fn partitions<'a, 'b, 'c, T: RefTuple<'c>>(
        &'a self,
        partition_size: usize,
    ) -> TablePartitions<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        TablePartitions {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            size: partition_size,
            curr: 0,
            end: self.len,
        }
    }

    pub unsafe fn partitions_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(
        &'a self,
        partition_size: usize,
    ) -> TablePartitionsMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        TablePartitionsMut {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            size: partition_size,
            curr: 0,
            end: self.len,
        }
    }

    pub unsafe fn iter<'a, 'b, 'c, T: RefTuple<'c>>(&'a self) -> TableIter<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        TableIter {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.len,
        }
    }


    pub unsafe fn iter_range<'a, 'b, 'c, T: RefTuple<'c>>(&'a self, range: Range<usize>) -> TableIter<'b, 'c, T>
        where
            'a: 'b,
            'b: 'c,
    {
        TableIter {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            curr: cmp::min(range.start, self.len),
            end: cmp::min(range.end, self.len),
        }
    }

    pub unsafe fn iter_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(&'a self) -> TableIterMut<'b, 'c, T>
    where
        'a: 'b,
        'b: 'c,
    {
        TableIterMut {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            curr: 0,
            end: self.len,
        }
    }

    pub unsafe fn iter_range_mut<'a, 'b, 'c, T: BorrowTuple<'c>>(&'a self, range: Range<usize>) -> TableIterMut<'b, 'c, T>
        where
            'a: 'b,
            'b: 'c,
    {
        TableIterMut {
            phantom: PhantomData,
            ptr: self.as_mut_ptr::<T::ValueType>(),
            curr: cmp::min(range.start, self.len),
            end: cmp::min(range.end, self.len),
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        self.columns.iter_mut().for_each(|c| unsafe {
            c.manually_drop(self.len);
        });
    }
}

pub struct TablePartitions<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    size: usize,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: RefTuple<'b>> Iterator for TablePartitions<'a, 'b, T>
where
    'a: 'b,
{
    type Item = TablePartition<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            Some(TablePartition {
                phantom: PhantomData,
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

pub struct TablePartitionsMut<'a, 'b, T: BorrowTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    size: usize,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: BorrowTuple<'b>> TablePartitionsMut<'a, 'b, T> where 'a: 'b {
    pub fn empty() -> Self {
        Self {
            phantom: PhantomData,
            ptr: <T::ValueType as ValueTuple>::PtrType::null_ptr(),
            size: 0,
            curr: 0,
            end: 0
        }
    }
}

impl<'a, 'b, T: BorrowTuple<'b>> Iterator for TablePartitionsMut<'a, 'b, T>
where
    'a: 'b,
{
    type Item = TablePartitionMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            Some(TablePartitionMut {
                phantom: PhantomData,
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

pub struct TablePartition<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    start: usize,
    end: usize,
}

impl<'a, 'b, T: RefTuple<'b>> TablePartition<'a, 'b, T>
where
    'a: 'b,
{
    pub fn iter<'c>(&self) -> TableIter<'c, 'b, T>
    where
        'c: 'b,
    {
        TableIter {
            phantom: PhantomData,
            ptr: self.ptr,
            curr: self.start,
            end: self.end,
        }
    }
}

pub struct TablePartitionMut<'a, 'b, T: BorrowTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    start: usize,
    end: usize,
}

unsafe impl<'a, 'b, T: BorrowTuple<'b>> Send for TablePartitionMut<'a, 'b, T> where 'a: 'b {}

impl<'a, 'b, T: BorrowTuple<'b>> TablePartitionMut<'a, 'b, T>
where
    'a: 'b,
{
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn iter<'c>(&mut self) -> TableIterMut<'c, 'b, T>
    where
        'c: 'b,
    {
        TableIterMut {
            phantom: PhantomData,
            ptr: self.ptr,
            curr: self.start,
            end: self.end,
        }
    }
}

pub struct TableIter<'a, 'b, T: RefTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: RefTuple<'b>> Iterator for TableIter<'a, 'b, T>
where
    'a: 'b,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            unsafe { Some(T::deref(self.ptr.add(self.curr))) }
        } else {
            None
        };
        self.curr += 1;
        result
    }
}

pub struct TableIterMut<'a, 'b, T: BorrowTuple<'b>>
where
    'a: 'b,
{
    phantom: PhantomData<(&'a Table, T)>,
    ptr: <T::ValueType as ValueTuple>::PtrType,
    curr: usize,
    end: usize,
}

impl<'a, 'b, T: BorrowTuple<'b>> Iterator for TableIterMut<'a, 'b, T>
where
    'a: 'b,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr < self.end {
            unsafe { Some(T::deref(self.ptr.add(self.curr))) }
        } else {
            None
        };
        self.curr += 1;
        result
    }
}

pub struct DebugTableEntry<'a> {
    pub(crate) table: &'a Table,
    pub(crate) index: usize,
}

impl<'a> Debug for DebugTableEntry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug_tuple = f.debug_tuple("TableEntry");
        self.table.columns.iter().for_each(|c| unsafe {
            debug_tuple.field(&c.debug_entry(self.index));
        });
        debug_tuple.finish()
    }
}

impl Debug for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug_list = f.debug_list();
        (0..self.len).for_each(|i| {
            debug_list.entry(&DebugTableEntry {
                table: self,
                index: i,
            });
        });
        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::table::Table;

    #[test]
    fn test() {
        unsafe {
            let mut t = Table::new::<(i32, i64, f32)>();
            for i in 0..25 {
                t.push((i, 1i64, 3.3f32));
            }
            for p in t.partitions::<(&i32, &f32)>(8) {
                println!("PARTITION START");
                for (x, y) in p.iter() {
                    println!("{} {}", x, y);
                }
                println!("PARTITION END");
            }
            // println!("{:?}", t);
        }
    }
}
