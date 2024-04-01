use crate::tuple::{ptr::PtrTuple, value::ValueTuple};

pub trait RefType<'a> {
    type ValueType: 'static;
    unsafe fn deref(ptr: *mut Self::ValueType) -> Self;
}

impl<'a, T: 'static> RefType<'a> for &'a T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &*ptr
    }
}

pub trait BorrowType<'a> {
    type ValueType: 'static;
    unsafe fn deref(ptr: *mut Self::ValueType) -> Self;
}

impl<'a, T: 'static> BorrowType<'a> for &'a T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &*ptr
    }
}

impl<'a, T: 'static> BorrowType<'a> for &'a mut T {
    type ValueType = T;

    unsafe fn deref(ptr: *mut Self::ValueType) -> Self {
        &mut *ptr
    }
}

pub trait TupleAdd<T> {
    type Result;
}

pub trait TupleAddMut<T> {
    type Result;
}

pub trait TupleAddRef<T> {
    type Result;
}

pub trait RefTuple<'a> {
    type ValueType: ValueTuple;
    unsafe fn deref(ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self;
}

impl<'a> RefTuple<'a> for () {
    type ValueType = ();

    unsafe fn deref(_ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self {
        ()
    }
}

pub trait BorrowTuple<'a> {
    type ValueType: ValueTuple;
    type ReadType: ValueTuple;
    type WriteType: ValueTuple;

    unsafe fn deref(ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self;
}

impl<'a> BorrowTuple<'a> for () {
    type ReadType = ();
    type ValueType = ();
    type WriteType = ();

    unsafe fn deref(_ptr: <Self::ValueType as ValueTuple>::PtrType) -> Self {
        ()
    }
}

#[cfg(test)]
mod tests {
    use crate::tuple::borrow::{BorrowTuple, RefTuple, TupleAdd, TupleAddMut, TupleAddRef};

    fn display_name<T>() {
        println!("{}", std::any::type_name::<T>());
    }
    #[test]
    fn test() {
        display_name::<<(i32, i64) as TupleAdd<String>>::Result>();
    }

    #[test]
    fn test2() {
        display_name::<<() as TupleAddRef<&String>>::Result>();
        display_name::<<() as TupleAddRef<&mut String>>::Result>();
        display_name::<<(&i32, &i64) as TupleAddRef<&String>>::Result>();
        display_name::<<(&i32, &i64) as TupleAddRef<&mut String>>::Result>();
        display_name::<<(&mut i32, &mut i64) as TupleAddMut<&mut String>>::Result>();
        display_name::<<(&mut i32, &mut i64) as TupleAddMut<&String>>::Result>();
    }

    #[test]
    fn test3() {
        let mut a: i32 = 1;
        let mut b: i64 = 3;
        let mut c: f32 = 1.253;
        let d = (&mut a as *mut i32, &mut b as *mut i64, &mut c as *mut f32);
        unsafe {
            let (x, y, z) = <(&i32, &i64, &f32) as RefTuple>::deref(d);
            println!("{} {} {}", x, y, z);
        }
    }
}
