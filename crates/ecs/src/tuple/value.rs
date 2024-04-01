use std::any::TypeId;

use crate::tuple::ptr::PtrTuple;

pub trait ValueTuple {
    type PtrType: PtrTuple;
    type TypeIdArray: AsMut<[TypeId]>
        + AsRef<[TypeId]>
        + Into<Box<[TypeId]>>
        + IntoIterator<Item = TypeId>;
    fn type_ids() -> Self::TypeIdArray;
    unsafe fn write(self, ptr: Self::PtrType);
    unsafe fn read(ptr: Self::PtrType) -> Self;
}

impl ValueTuple for () {
    type PtrType = ();
    type TypeIdArray = [TypeId; 0];

    fn type_ids() -> Self::TypeIdArray {
        []
    }

    unsafe fn write(self, _ptr: Self::PtrType) {}

    unsafe fn read(_ptr: Self::PtrType) -> Self {
        ()
    }
}
