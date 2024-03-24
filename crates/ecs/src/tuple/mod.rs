use std::{any::TypeId, fmt::Debug};

use serverx_macros::tuple_impl;

use crate::{
    component::Component,
    storage::blob::BlobStorage,
    tuple::{ptr::PtrTuple, type_tuple::TypeTuple},
};

pub mod ptr;
pub mod type_tuple;

pub trait ComponentTuple: TypeTuple {
    type PtrType: PtrTuple;
    type BlobArray: AsMut<[BlobStorage]>
        + AsRef<[BlobStorage]>
        + Into<Box<[BlobStorage]>>
        + IntoIterator<Item = BlobStorage>;
    unsafe fn write(self, ptr: Self::PtrType);
    fn blobs() -> Self::BlobArray;
}

impl ComponentTuple for () {
    type BlobArray = [BlobStorage; 0];
    type PtrType = ();

    unsafe fn write(self, ptr: Self::PtrType) {}

    fn blobs() -> Self::BlobArray {
        []
    }
}

pub trait ComponentBorrowType<'a> {
    type ValueType: Component;
    unsafe fn deref(ptr: *mut Self::ValueType) -> Self;
}

impl<'a, T: Component> ComponentBorrowType<'a> for &'a mut T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &mut *ptr
    }
}

impl<'a, T: Component> ComponentBorrowType<'a> for &'a T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &*(ptr as *const T)
    }
}

pub trait ComponentRefType<'a> {
    type ValueType: Component;
    unsafe fn deref(ptr: *mut Self::ValueType) -> Self;
}

impl<'a, T: Component> ComponentRefType<'a> for &'a T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &*(ptr as *const T)
    }
}

pub trait ComponentBorrowTuple<'a> {
    type ValueType: ComponentTuple;
    type ReadType: ComponentTuple;
    type WriteType: ComponentTuple;
    unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self;
}

impl<'a> ComponentBorrowTuple<'a> for () {
    type ReadType = ();
    type ValueType = ();
    type WriteType = ();

    unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self {
        ()
    }
}

impl<'a, T: ComponentBorrowType<'a>> ComponentBorrowTuple<'a> for (T,)
where
    (): ComponentTupleAddRef<T>,
    (): ComponentTupleAddMut<T>,
{
    type ReadType = <() as ComponentTupleAddRef<T>>::ValueType;
    type ValueType = (T::ValueType,);
    type WriteType = <() as ComponentTupleAddMut<T>>::ValueType;

    unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self {
        (T::deref(ptr.0),)
    }
}

pub trait ComponentRefTuple<'a> {
    type ValueType: ComponentTuple;
    unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self;
}

impl<'a> ComponentRefTuple<'a> for () {
    type ValueType = ();

    unsafe fn deref(ptr: <Self::ValueType as ComponentTuple>::PtrType) -> Self {
        ()
    }
}

pub trait ComponentTupleAdd<T: Component> {
    type ValueType: ComponentTuple;
}

impl<T: Component> ComponentTupleAdd<T> for ()
where
    (T,): ComponentTuple,
{
    type ValueType = (T,);
}

pub trait ComponentTupleAddRef<T> {
    type ValueType: ComponentTuple;
}

impl<'a, T: Component> ComponentTupleAddRef<&'a T> for ()
where
    (T,): ComponentTuple,
{
    type ValueType = (T,);
}

impl<'a, T: Component> ComponentTupleAddRef<&'a mut T> for () {
    type ValueType = ();
}

pub trait ComponentTupleAddMut<T> {
    type ValueType: ComponentTuple;
}

impl<'a, T: Component> ComponentTupleAddMut<&'a T> for () {
    type ValueType = ();
}

impl<'a, T: Component> ComponentTupleAddMut<&'a mut T> for ()
where
    (T,): ComponentTuple,
{
    type ValueType = (T,);
}

tuple_impl!(10);

#[cfg(test)]
mod tests {
    use std::{any::type_name, fmt::Debug, mem::MaybeUninit};

    use crate::tuple::{
        ComponentBorrowTuple, ComponentTuple, ComponentTupleAdd, ComponentTupleAddMut,
        ComponentTupleAddRef,
    };

    #[test]
    fn test0() {
        println!(
            "{:?}",
            type_name::<<(i8, i16, i32) as ComponentTupleAdd<i64>>::ValueType>()
        );
        println!(
            "{:?}",
            type_name::<<(i8, i16, i32) as ComponentTupleAddRef<&i64>>::ValueType>()
        );
        println!(
            "{:?}",
            type_name::<<(i8, i16, i32) as ComponentTupleAddRef<&mut i64>>::ValueType>()
        );
        println!(
            "{:?}",
            type_name::<<(i8, i16, i32) as ComponentTupleAddMut<&i64>>::ValueType>()
        );
        println!(
            "{:?}",
            type_name::<<(i8, i16, i32) as ComponentTupleAddMut<&mut i64>>::ValueType>()
        );
    }

    fn print_rw_type<'a, T: ComponentBorrowTuple<'a>>() {
        println!("{:?}", type_name::<T::ValueType>());
        println!("{:?}", type_name::<T::ReadType>());
        println!("{:?}", type_name::<T::WriteType>());
    }

    #[test]
    fn test1() {
        print_rw_type::<(&mut i32, &i64, &f32, &mut f64)>();
    }

    #[test]
    fn test() {
        let mut a = MaybeUninit::<i32>::uninit();
        let mut b = MaybeUninit::<i64>::uninit();
        let mut c = MaybeUninit::<f32>::uninit();
        let ptr = (a.as_mut_ptr(), b.as_mut_ptr(), c.as_mut_ptr());
        unsafe {
            // (12i32, 16i64, 1.32f32).write(ptr);
            println!(
                "{} {} {}",
                a.assume_init(),
                b.assume_init(),
                c.assume_init()
            );
        }
    }
}
